use vek::Vec3;

#[derive(Debug)]
pub struct Mesh {
    indices: Vec<u32>,
    /// The position of each vertex of the model
    positions: Vec<Vec3<f32>>,
    /// The normal of each vertex of the model
    normals: Vec<Vec3<f32>>,
}

impl Mesh {
    pub fn new(mesh: tobj::Mesh) -> Self {
        Self {
            indices: mesh.indices,
            positions: mesh.positions.chunks(3).map(|sl| Vec3::from_slice(sl)).collect(),
            normals: mesh.normals.chunks(3).map(|sl| Vec3::from_slice(sl)).collect(),
        }
    }

    /// Returns the indices of this mesh
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Returns the position for the given index
    pub fn position(&self, index: usize) -> Vec3<f32> {
        self.positions[index]
    }

    /// Returns the normal for the given index
    pub fn normal(&self, index: usize) -> Vec3<f32> {
        self.normals[index]
    }
}
