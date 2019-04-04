use std::path::Path;
use std::rc::Rc;
use tobj;

use crate::geometry::Mesh;
use crate::material::Material;

pub fn load_file(filepath: &str) -> Vec<Mesh> {
    let (meshes, materials) = tobj::load_obj(&Path::new(filepath)).unwrap();
    let materials: Vec<_> = materials
        .into_iter()
        .map(|mat| Rc::new(Material::from(mat)))
        .collect();
    let meshes = meshes
        .into_iter()
        .map(|model| Mesh::from_obj(model.mesh, &materials))
        .collect();
    meshes
}
