use std::path::Path;
use std::sync::Arc;

use tobj;

use crate::rayon_polyfill::*;
use crate::model::{Mesh, Material, Model};

pub fn load_file(path: impl AsRef<Path>) -> Result<Model, tobj::LoadError> {
    let (meshes, materials) = tobj::load_obj(path.as_ref())?;
    let materials: Vec<_> = materials
        .into_par_iter()
        .map(|mat| Arc::new(Material::from(mat)))
        .collect();
    let meshes = meshes
        .into_par_iter()
        .map(|model| Mesh::from_obj(model.mesh, &materials))
        .collect();
    Ok(Model {meshes})
}
