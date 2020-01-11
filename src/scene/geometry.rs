use std::sync::Arc;

use vek::Vec3;

use super::Material;

#[derive(Debug, Clone)]
pub struct Geometry {
    /// The name of the primitive (possibly empty), or None if the 3D file this was loaded from does
    /// not support mesh names
    pub name: Option<String>,
    /// The indexes that represent the triangles of this geometry
    pub indices: Vec<u32>,
    /// The position of each vertex of the geometry
    pub positions: Vec<Vec3<f32>>,
    /// The normal of each vertex of the geometry
    pub normals: Vec<Vec3<f32>>,
    /// The material associated with this geometry
    pub material: Arc<Material>,
}

impl Geometry {
    pub fn from_obj(model: tobj::Model, materials: &[Arc<Material>]) -> Self {
        let tobj::Model {name, mesh} = model;

        Self {
            name: Some(name),
            indices: mesh.indices,
            positions: mesh.positions.chunks(3).map(|sl| Vec3::from_slice(sl)).collect(),
            normals: mesh.normals.chunks(3).map(|sl| Vec3::from_slice(sl)).collect(),
            material: mesh.material_id.map(|id| materials[id].clone()).unwrap_or_default(),
        }
    }

    pub fn from_gltf(
        prim: gltf::Primitive,
        materials: &[Arc<Material>],
        buffers: &[gltf::buffer::Data],
    ) -> Self {
        assert_eq!(prim.mode(), gltf::mesh::Mode::Triangles, "Non-triangle meshes are not supported");

        // glTF primitives do not have names
        let name = None;

        let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));
        let indices = reader.read_indices()
            .expect("Unable to read index buffer from glTF geometry")
            .into_u32()
            .collect();
        let positions: Vec<_> = reader.read_positions()
            .expect("Unable to read vertex positions from glTF geometry")
            .map(Vec3::from)
            .collect();
        let normals: Vec<_> = reader.read_normals()
            .expect("Unable to read vertex normals from glTF geometry")
            .map(Vec3::from)
            .collect();

        // index() returns None if the material is the glTF default material
        // See: https://github.com/KhronosGroup/glTF/tree/92f59a0dbefe2d54cff38dba103cd70462cc778b/specification/2.0#default-material
        let material = prim.material().index()
            .map(|id| materials[id].clone())
            .unwrap_or_default();

        // Not handling optional normals yet
        assert_eq!(
            positions.len(),
            normals.len(),
            "glTF geometry must have exactly as many positions as normals"
        );

        Self {name, indices, positions, normals, material}
    }
}
