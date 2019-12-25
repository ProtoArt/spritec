use std::sync::{Arc, Mutex};

use crate::model::Model;
use crate::query3d::GeometryQuery;

use super::{File, QueryBackend, QueryError};

/// Represents one or more 3D files
///
/// With multiple 3D files, each file can be treated as an animation frame, indexed by frame
/// number only.
#[derive(Debug)]
pub struct MultiFile {
    files: Vec<Arc<Mutex<File>>>,
}

impl MultiFile {
    pub fn new(files: Vec<Arc<Mutex<File>>>) -> Self {
        Self {files}
    }
}

impl QueryBackend for MultiFile {
    fn query_geometry(&mut self, query: GeometryQuery) -> Result<Vec<&Model>, QueryError> {
        unimplemented!()
    }
}
