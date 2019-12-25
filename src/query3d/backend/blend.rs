use crate::query3d::GeometryQuery;

use crate::model::Model;
use super::{QueryBackend, QueryError};

#[derive(Debug)]
pub struct BlendFile {
}

impl QueryBackend for BlendFile {
    fn query_geometry(&mut self, query: GeometryQuery) -> Result<Vec<&Model>, QueryError> {
        unimplemented!()
    }
}
