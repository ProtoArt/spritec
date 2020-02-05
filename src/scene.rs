mod mesh;
mod geometry;
mod material;
mod node;
mod node_tree;
mod camera_type;
mod light_type;

pub use mesh::*;
pub use geometry::*;
pub use material::*;
pub use node::*;
pub use node_tree::*;
pub use camera_type::*;
pub use light_type::*;

#[derive(Debug, Clone)]
pub struct Scene {
    /// The name of the scene (possibly empty), or None if the 3D file this was loaded from does
    /// not support scene names
    pub name: Option<String>,
    /// The root nodes of the scene
    pub roots: Vec<NodeId>,
}

impl Scene {
    pub fn from_gltf(scene: gltf::Scene) -> Self {
        Self {
            name: Some(scene.name().unwrap_or("").to_string()),
            roots: scene.nodes()
                .map(|node| NodeId::from_gltf(&node))
                .collect(),
        }
    }
}
