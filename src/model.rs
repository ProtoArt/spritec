mod geometry;
mod material;

pub use geometry::*;
pub use material::*;

#[derive(Debug, Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}
