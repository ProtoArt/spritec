use std::rc::Rc;

use vek::Vec3;

use crate::material::Material;

#[derive(Debug)]
pub struct Mesh {
    indices: Vec<u32>,
    /// The position of each vertex of the model
    positions: Vec<Vec3<f32>>,
    /// The normal of each vertex of the model
    normals: Vec<Vec3<f32>>,
    /// The material associated with this mesh (if any)
    material: Rc<Material>,
}

impl Mesh {
    pub fn new(mesh: tobj::Mesh, materials: &[Rc<Material>]) -> Self {
        Self {
            indices: mesh.indices,
            positions: mesh.positions.chunks(3).map(|sl| Vec3::from_slice(sl)).collect(),
            normals: mesh.normals.chunks(3).map(|sl| Vec3::from_slice(sl)).collect(),
            material: mesh.material_id.map(|id| materials[id].clone()).unwrap_or_default(),
        }
    }

    pub fn from_gltf(
        indices: Vec<u32>,
        positions: Vec<[f32; 3]>,
        normals: Vec<[f32; 3]>,
        material: Rc<Material>
    ) -> Self {
        Self {
            indices: indices,
            positions: positions.iter().map(|data| Vec3::new(data[0], data[1], data[2])).collect(),
            normals: normals.iter().map(|data| Vec3::new(data[0], data[1], data[2])).collect(),
            material: material
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

    /// Returns the material associated with this mesh
    pub fn material(&self) -> &Material {
        &self.material
    }
}
