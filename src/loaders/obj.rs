use std::path::Path;
use std::sync::Arc;
use std::io::BufRead;

use tobj;

use crate::rayon_polyfill::*;
use crate::model::{Mesh, Material, Model};

pub fn load_file(path: impl AsRef<Path>) -> Result<Model, tobj::LoadError> {
    let (meshes, materials) = tobj::load_obj(path.as_ref())?;
    from_obj(meshes, materials)
}

pub fn from_reader<R: BufRead>(
    reader: &mut R,
    material_loader: impl Fn(&Path) -> tobj::MTLLoadResult,
) -> Result<Model, tobj::LoadError> {
    let (meshes, materials) = tobj::load_obj_buf(reader, material_loader)?;
    from_obj(meshes, materials)
}

#[inline(always)]
fn from_obj(meshes: Vec<tobj::Model>, materials: Vec<tobj::Material>) -> Result<Model, tobj::LoadError> {
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
