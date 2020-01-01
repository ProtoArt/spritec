use std::path::Path;
use std::sync::Arc;

use rayon::iter::{ParallelIterator, IntoParallelIterator};

use crate::camera::Camera;
use crate::light::DirectionalLight;
use crate::model::{Mesh, Material, Model};
use crate::renderer::{Display, RenderModel, RenderMesh};
use crate::query3d::{GeometryQuery, GeometryFilter, AnimationQuery, CameraQuery, LightQuery};

use super::{QueryBackend, QueryError};

/// Represents a single OBJ file
#[derive(Debug)]
pub struct ObjFile {
    // This representation is sufficient for GeometryFilter::Scene, but it will need to
    // change once we add in more advanced filtering (e.g. by name)

    model: Arc<Model>,
    /// The version of this model lazily uploaded to the GPU
    render_model: Option<Arc<RenderModel>>,
}

impl ObjFile {
    /// Opens a OBJ file
    pub fn open(path: &Path) -> Result<Self, tobj::LoadError> {
        let (meshes, materials) = tobj::load_obj(path)?;

        let materials: Vec<_> = materials
            .into_par_iter()
            .map(|mat| Arc::new(Material::from(mat)))
            .collect();

        let meshes = meshes
            .into_par_iter()
            .map(|model| Mesh::from_obj(model.mesh, &materials))
            .collect();

        Ok(Self {
            model: Arc::new(Model {meshes}),
            render_model: None,
        })
    }
}

impl QueryBackend for ObjFile {
    fn query_geometry(&mut self, query: &GeometryQuery, display: &Display) -> Result<Vec<Arc<RenderModel>>, QueryError> {
        let GeometryQuery {models, animation} = query;

        // OBJ files do not support animations
        if let Some(AnimationQuery {name, ..}) = animation {
            return Err(QueryError::UnknownAnimation {name: name.clone()});
        }

        use GeometryFilter::*;
        match models {
            Scene {name: None} => match &self.render_model {
                Some(render_model) => Ok(vec![render_model.clone()]),
                None => {
                    let render_model = Arc::new(RenderModel {
                        meshes: self.model.meshes.iter()
                            .map(|mesh| RenderMesh::new(display, &*mesh))
                            .collect::<Result<Vec<_>, _>>()?,
                    });
                    self.render_model = Some(render_model.clone());

                    Ok(vec![render_model])
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
            // OBJ files do not contain any named scenes
            CameraQuery::FirstInScene {name: Some(name)} => Err(QueryError::UnknownScene {
                name: name.clone(),
            }),
        }
    }

    fn query_lights(&mut self, query: &LightQuery) -> Result<Vec<Arc<DirectionalLight>>, QueryError> {
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
