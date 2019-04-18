use std::path::Path;
use std::sync::Arc;

use rayon::iter::{ParallelIterator, IntoParallelIterator};
use tobj;

use crate::geometry::Mesh;
use crate::material::Material;

pub fn load_file(path: impl AsRef<Path>) -> Result<Vec<Mesh>, tobj::LoadError> {
    let (meshes, materials) = tobj::load_obj(path.as_ref()).unwrap();
    let materials: Vec<_> = materials
        .into_par_iter()
        .map(|mat| Arc::new(Material::from(mat)))
        .collect();
    let meshes = meshes
        .into_par_iter()
        .map(|model| Mesh::from_obj(model.mesh, &materials))
        .collect();
    Ok(meshes)
}
