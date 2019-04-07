use std::rc::Rc;

use crate::geometry::Mesh;
use crate::material::Material;

pub fn load_file(filepath: &str) -> Vec<Mesh> {
    let (document, buffers, _) = gltf::import(filepath).expect("Could not open Gltf file");

    let mut ret: Vec<Mesh> = Vec::new();

    // Load all the materials first, this assumes that the material index
    // that primitive refers to is loaded in the same order as document.materials()
    let materials: Vec<_> = document
        .materials()
        .map(|material| Rc::new(Material::from(material)))
        .collect();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            ret.push(Mesh::from_gltf(&buffers, &primitive, &materials));
        }
    }

    ret
}
