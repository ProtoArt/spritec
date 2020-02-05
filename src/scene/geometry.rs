use std::sync::Arc;

use crate::math::{Vec3, Vec4};

use super::Material;

#[derive(Debug, Clone)]
pub struct Geometry {
    /// The name of the primitive (possibly empty), or None if the 3D file this was loaded from does
    /// not support mesh names
    pub name: Option<String>,
    /// The indexes that represent the triangles of the geometry
    pub indices: Vec<u32>,
    /// The position of each vertex of the geometry
    pub positions: Vec<Vec3>,
    /// The normal of each vertex of the geometry
    pub normals: Vec<Vec3>,
    /// The joint indexes (up to 4) that affect each vertex of the geometry.
    /// These indexes map into the `joints` array in `skin`.
    ///
    /// If there are less than 4, this array will still contain valid indexes. The corresponding
    /// weights will be 0.0.
    pub joint_influences: Option<Vec<[u16; 4]>>,
    /// The weights of each joint (up to 4) for each vertex of the geometry
    ///
    /// If the joints are [1, 2, 3, 4], the weights are respectively:
    /// * `weights.x` for joint 1
    /// * `weights.y` for joint 2
    /// * `weights.z` for joint 3
    /// * `weights.w` for joint 4
    pub weights: Option<Vec<Vec4>>,
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
            joint_influences: None,
            weights: None,
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

        // We only support JOINTS_0 and WEIGHTS_0 (implies max 4 joint influences per vertex)
        let joint_influences = reader.read_joints(0)
            .map(|joints| joints.into_u16().collect::<Vec<_>>());
        let weights = reader.read_weights(0)
            .map(|weights| weights.into_f32().map(Vec4::from).collect::<Vec<_>>());

        // index() returns None if the material is the glTF default material
        // See: https://github.com/KhronosGroup/glTF/tree/92f59a0dbefe2d54cff38dba103cd70462cc778b/specification/2.0#default-material
        let material = prim.material().index()
            .map(|id| materials[id].clone())
            .unwrap_or_default();

        // Not handling optional normals yet
        assert_eq!(positions.len(), normals.len(),
            "glTF geometry must have exactly as many normals as vertices");
        if let Some(joint_influences) = &joint_influences {
            assert_eq!(positions.len(), joint_influences.len(),
                "glTF geometry must have exactly as many vertex joint influences as vertices");
        }
        if let Some(weights) = &weights {
            assert_eq!(positions.len(), weights.len(),
                "glTF geometry must have exactly as many vertex weights as vertices");
        }

        Self {name, indices, positions, normals, joint_influences, weights, material}
    }
}
