use std::sync::Arc;
use std::path::Path;

use crate::geometry::Mesh;
use crate::material::Material;

pub fn load_file(filepath: impl AsRef<Path>) -> Result<Vec<Mesh>, gltf::Error> {
    let (document, buffers, _) = gltf::import(filepath)?;

    // Load all the materials first, this assumes that the material index
    // that primitive refers to is loaded in the same order as document.materials()
    let materials: Vec<_> = document.materials()
        .map(|material| Arc::new(Material::from(material)))
        .collect();

    let mut meshes = Vec::new();
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            meshes.push(Mesh::from_gltf(&buffers, &primitive, &materials));
        }
    }

    Ok(meshes)
}
