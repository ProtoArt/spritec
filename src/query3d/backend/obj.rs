use std::path::Path;

use crate::model::Model;
use crate::query3d::GeometryQuery;

use super::{QueryBackend, QueryError};

/// Represents a single OBJ file
#[derive(Debug)]
pub struct ObjFile {
}

impl ObjFile {
    /// Opens a OBJ file
    pub fn open(path: &Path) -> Result<Self, tobj::LoadError> {
        unimplemented!()
    }
}

impl QueryBackend for ObjFile {
    fn query_geometry(&mut self, query: GeometryQuery) -> Result<Vec<&Model>, QueryError> {
        unimplemented!()
    }
}
