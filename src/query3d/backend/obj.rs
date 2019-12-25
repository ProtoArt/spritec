use std::path::Path;
use std::sync::Arc;

use rayon::iter::{ParallelIterator, IntoParallelIterator};

use crate::model::{Mesh, Material, Model};
use crate::query3d::{GeometryQuery, GeometryFilter, AnimationQuery};

use super::{QueryBackend, QueryError};

/// Represents a single OBJ file
#[derive(Debug)]
pub struct ObjFile {
    // This representation is sufficient for GeometryFilter::Scene, but it will need to
    // change once we add in more advanced filtering (e.g. by name)
    model: Model,
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
            model: Model {meshes},
        })
    }
}

impl QueryBackend for ObjFile {
    fn query_geometry(&mut self, query: GeometryQuery) -> Result<Vec<&Model>, QueryError> {
        let GeometryQuery {models, animation} = query;

        // OBJ files do not support animations
        if let Some(AnimationQuery {name, ..}) = animation {
            return Err(QueryError::UnknownAnimation {name});
        }

        use GeometryFilter::*;
        match models {
            Scene {name: None} => Ok(vec![&self.model]),
            // OBJ files do not contain any named scenes
            Scene {name: Some(name)} => Err(QueryError::UnknownScene {name}),
        }
    }
}
