use std::sync::Arc;

use super::{Material, Geometry};

#[derive(Debug, Clone)]
pub struct Mesh {
    /// The name of the mesh (possibly empty), or None if the 3D file this was loaded from does
    /// not support mesh names
    pub name: Option<String>,
    /// The geometry stored in this mesh and their associated materials
    pub geometry: Vec<Geometry>,
}

impl Mesh {
    pub fn from_obj(models: Vec<tobj::Model>, materials: &[Arc<Material>]) -> Self {
        Self {
            // Currently, tobj doesn't fully model the hierarchy of objects and groups in OBJ files.
            // That means that the object name isn't actually accessible always. Even when it is,
            // it isn't possible to distinguish between objects and their groups.
            // See: https://github.com/Twinklebear/tobj/issues/15
            name: None,
            geometry: models.into_iter()
                .map(|model| Geometry::from_obj(model, materials))
                .collect(),
        }
    }

    pub fn from_gltf(
        mesh: gltf::Mesh,
        materials: &[Arc<Material>],
        buffers: &[gltf::buffer::Data],
    ) -> Self {
        Self {
            name: Some(mesh.name().unwrap_or("").to_string()),
            geometry: mesh.primitives()
                .map(|prim| Geometry::from_gltf(prim, materials, buffers))
                .collect(),
        }
    }
}
