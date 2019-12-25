use std::path::Path;

use crate::model::Model;
use crate::query3d::GeometryQuery;

use super::{QueryBackend, QueryError};

/// Represents one or more OBJ files
///
/// With multiple OBJ files, each file can be treated as an animation frame, indexed by frame
/// number only.
#[derive(Debug)]
pub struct ObjFiles {
}

impl ObjFiles {
    /// Opens an OBJ file
    pub fn open(path: &Path) -> Result<Self, tobj::LoadError> {
        unimplemented!()
    }
}

impl QueryBackend for ObjFiles {
    fn query_geometry(&mut self, query: GeometryQuery) -> Result<Vec<&Model>, QueryError> {
        unimplemented!()
    }
}
