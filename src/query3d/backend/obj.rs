use std::path::Path;
use std::sync::Arc;

use rayon::iter::{ParallelIterator, IntoParallelIterator};

use crate::math::Mat4;
use crate::scene::{Mesh, Material};
use crate::renderer::{Display, ShaderGeometry, JointMatrixTexture, Camera, Light};
use crate::query3d::{GeometryQuery, GeometryFilter, AnimationQuery, CameraQuery, LightQuery};

use super::{QueryBackend, QueryError};

/// Represents a single OBJ file
#[derive(Debug)]
pub struct ObjFile {
    // This representation is sufficient for GeometryFilter::Scene, but it will need to
    // change once we add in more advanced filtering (e.g. by name)

    mesh: Mesh,
    /// The version of this model lazily uploaded to the GPU
    scene_geometry: Option<Arc<Vec<Arc<ShaderGeometry>>>>,
}

impl ObjFile {
    /// Opens a OBJ file
    pub fn open(path: &Path) -> Result<Self, tobj::LoadError> {
        let (models, materials) = tobj::load_obj(path)?;

        let materials: Vec<_> = materials.into_par_iter()
            .map(|mat| Arc::new(Material::from(mat)))
            .collect();

        Ok(Self {
            mesh: Mesh::from_obj(models, &materials),
            scene_geometry: None,
        })
    }
}

impl QueryBackend for ObjFile {
    fn query_geometry(&mut self, query: &GeometryQuery, display: &Display) -> Result<Arc<Vec<Arc<ShaderGeometry>>>, QueryError> {
        let GeometryQuery {models, animation} = query;

        // OBJ files do not support animations
        match animation {
            Some(AnimationQuery {name: Some(name), ..}) => {
                return Err(QueryError::UnknownAnimation {name: name.clone()});
            },
            Some(AnimationQuery {name: None, ..}) => {
                return Err(QueryError::NoAnimationFound);
            },
            _ => {},
        }

        use GeometryFilter::*;
        match models {
            Scene {name: None} => match &self.scene_geometry {
                Some(scene_geometry) => Ok(scene_geometry.clone()),
                None => {
                    // Default to a single identity matrix (makes it so that even if
                    // we accidentally index into the texture, we won't get UB)
                    //TODO: Find a way to cache this texture so we don't have to upload it over and over again
                    let joint_matrices_tex = Arc::new(JointMatrixTexture::identity(display)?);
                    let scene_geometry = Arc::new(self.mesh.geometry.iter()
                        .map(|geo| {
                            ShaderGeometry::new(display, geo, &joint_matrices_tex, Mat4::identity()).map(Arc::new)
                        })
                        .collect::<Result<Vec<_>, _>>()?);

                    if scene_geometry.is_empty() {
                        return Err(QueryError::NoGeometryFound);
                    }

                    self.scene_geometry = Some(scene_geometry.clone());

                    Ok(scene_geometry)
                },
            },
            // OBJ files do not contain any named scenes
            Scene {name: Some(name)} => Err(QueryError::UnknownScene {name: name.clone()}),
        }
    }

    fn query_camera(&mut self, query: &CameraQuery) -> Result<Arc<Camera>, QueryError> {
        // OBJ files do not support cameras
        // This code still does the work to produce useful errors
        match query {
            CameraQuery::FirstInScene {name: None} => Err(QueryError::NoCameraFound),
            CameraQuery::Named {name, scene: None} => Err(QueryError::UnknownCamera {name: name.clone()}),

            // OBJ files do not contain any named scenes
            CameraQuery::FirstInScene {name: Some(name)} |
            CameraQuery::Named {name: _, scene: Some(name)} => Err(QueryError::UnknownScene {
                name: name.clone(),
            }),
        }
    }

    fn query_lights(&mut self, query: &LightQuery) -> Result<Arc<Vec<Arc<Light>>>, QueryError> {
        // OBJ files do not support lights
        // This code still does the work to produce useful errors
        match query {
            LightQuery::Scene {name: None} => Err(QueryError::NoLightsFound),
            // OBJ files do not contain any named scenes
            LightQuery::Scene {name: Some(name)} => Err(QueryError::UnknownScene {
                name: name.clone(),
            }),
        }
    }
}
