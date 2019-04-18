use std::sync::Arc;

use vek::{Vec3, Mat4};

use crate::material::Material;

#[derive(Debug, Clone)]
pub struct Mesh {
    indices: Vec<u32>,
    /// The position of each vertex of the model
    positions: Vec<Vec3<f32>>,
    /// The normal of each vertex of the model
    normals: Vec<Vec3<f32>>,
    /// The material associated with this mesh (if any)
    material: Arc<Material>,
    /// The model transformation represents the center of the mesh, all vertices in the model are
    /// relative to this. This is also known as the "world" transformation.
    ///
    /// Model coordinates -> World coordinates
    ///
    /// If this is the identity matrix, then all vertices in the model are already in world
    /// coordinates.
    transform: Mat4<f32>,
}

impl Mesh {
    pub fn from_obj(mesh: tobj::Mesh, materials: &[Arc<Material>]) -> Self {
        Self {
            indices: mesh.indices,
            positions: mesh.positions.chunks(3).map(|sl| Vec3::from_slice(sl)).collect(),
            normals: mesh.normals.chunks(3).map(|sl| Vec3::from_slice(sl)).collect(),
            material: mesh.material_id.map(|id| materials[id].clone()).unwrap_or_default(),
            transform: Mat4::identity(),
        }
    }

    pub fn from_gltf(
        buffers: &[gltf::buffer::Data],
        primitive: &gltf::Primitive,
        materials: &[Arc<Material>],
    ) -> Self {
        // We're only dealing with triangle meshes
        assert_eq!(gltf::mesh::Mode::Triangles, primitive.mode(), "Not handling non-triangle glTF primitives");

        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let indices = reader
            .read_indices()
            .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>())
            .expect("Failed to read glTF indices");
        let positions: Vec<_> = reader.read_positions().expect("Failed to read glTF positions")
            .map(|data| Vec3::new(data[0], data[1], data[2]))
            .collect();
        let normals: Vec<_> = reader.read_normals().expect("Failed to read glTF normals")
            .map(|data| Vec3::new(data[0], data[1], data[2]))
            .collect();
        let material = primitive.material().index().map(|id| materials[id].clone()).unwrap_or_default();

        // Not handling optional normals yet
        assert_eq!(
            positions.len(),
            normals.len(),
            "Position vector and normals vector have different lengths"
        );

        Self {indices, positions, normals, material, transform: Mat4::identity()}
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

    /// Returns the model transformation for this mesh. All vertices of this mesh must have this
    /// applied to them in order to become world coordinates.
    pub fn transform(&self) -> Mat4<f32> {
        self.transform
    }
}
