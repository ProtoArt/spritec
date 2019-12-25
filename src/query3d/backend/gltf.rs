use std::sync::Arc;
use std::path::Path;

use crate::camera::Camera;
use crate::light::DirectionalLight;
use crate::model::Model;
use crate::query3d::{GeometryQuery, CameraQuery, LightQuery};

use super::{QueryBackend, QueryError};

/// Represents a single glTF file
#[derive(Debug)]
pub struct GltfFile {
}

impl GltfFile {
    /// Opens a glTF file
    pub fn open(path: &Path) -> Result<Self, tobj::LoadError> {
        unimplemented!()
    }
}

impl QueryBackend for GltfFile {
    fn query_geometry(&mut self, query: &GeometryQuery) -> Result<Vec<Arc<Model>>, QueryError> {
        unimplemented!()
    }

    fn query_camera(&mut self, query: &CameraQuery) -> Result<Arc<Camera>, QueryError> {
        unimplemented!()
    }

    fn query_lights(&mut self, query: &LightQuery) -> Result<Vec<Arc<DirectionalLight>>, QueryError> {
        unimplemented!()
    }
}
