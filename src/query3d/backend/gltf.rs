use std::path::Path;

use crate::model::Model;
use crate::query3d::GeometryQuery;

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
    fn query_geometry(&mut self, query: GeometryQuery) -> Result<Vec<&Model>, QueryError> {
        unimplemented!()
    }
}
