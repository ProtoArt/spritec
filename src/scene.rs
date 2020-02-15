mod camera_type;
mod geometry;
mod light_type;
mod material;
mod mesh;
mod node_tree;
mod node;
mod skin;

pub use camera_type::*;
pub use geometry::*;
pub use light_type::*;
pub use material::*;
pub use mesh::*;
pub use node_tree::*;
pub use node::*;
pub use skin::*;

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
