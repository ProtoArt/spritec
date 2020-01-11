mod mesh;
mod geometry;
mod material;
mod node;
mod camera_type;

pub use mesh::*;
pub use geometry::*;
pub use material::*;
pub use node::*;
pub use camera_type::*;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Scene {
    /// The name of the scene (possibly empty), or None if the 3D file this was loaded from does
    /// not support scene names
    pub name: Option<String>,
    /// The root nodes of the scene
    pub roots: Vec<Arc<Node>>,
}

impl Scene {
    pub fn from_gltf(
        scene: gltf::Scene,
        meshes: &[Arc<Mesh>],
        cameras: &[Arc<CameraType>],
    ) -> Self {
        Self {
            name: Some(scene.name().unwrap_or("").to_string()),
            roots: scene.nodes()
                .map(|node| Arc::new(Node::from_gltf(node, meshes, cameras)))
                .collect(),
        }
    }
}
