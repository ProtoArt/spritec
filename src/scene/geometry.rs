use std::sync::Arc;

use crate::math::{Vec2, Vec3, Vec4};

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
    /// The texture coordinates of each vertex of the geometry
    pub tex_coords: Option<Vec<Vec2>>,
    /// The joint indexes (up to 4) that affect each vertex of the geometry.
    /// These indexes map into the `joints` array in the `Skin` data applied to this geometry.
    ///
    /// If there are less than 4, this array will still contain valid indexes. The corresponding
    /// weights will be 0.0.
    ///
    /// If this field is None, `joint_weights` will also be None.
    pub joint_influences: Option<Vec<[u32; 4]>>,
    /// The weight of each influencing joint (up to 4) for each vertex of the geometry
    ///
    /// If the joint_influences for a vertex are [1, 2, 3, 4], the weights are respectively:
    /// * `weights.x` for joint 1
    /// * `weights.y` for joint 2
    /// * `weights.z` for joint 3
    /// * `weights.w` for joint 4
    ///
    /// If this field is None, `joint_influences` will also be None.
    pub joint_weights: Option<Vec<Vec4>>,
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
            tex_coords: None,
            joint_influences: None,
            joint_weights: None,
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
        // We only support TEXCOORD_0
        let tex_coords: Option<Vec<_>> = reader.read_tex_coords(0)
            .map(|tex_coords| tex_coords.into_f32().map(Vec2::from).collect());

        // We only support JOINTS_0 and WEIGHTS_0 (implies max 4 joint influences per vertex)
        let into_u32 = |[a, b, c, d]: [u16; 4]| [a as u32, b as u32, c as u32, d as u32];
        let joint_influences = reader.read_joints(0)
            .map(|joints| joints.into_u16().map(into_u32).collect::<Vec<_>>());
        let joint_weights = reader.read_weights(0)
            .map(|joint_weights| joint_weights.into_f32().map(Vec4::from).collect::<Vec<_>>());

        // index() returns None if the material is the glTF default material
        // See: https://github.com/KhronosGroup/glTF/tree/92f59a0dbefe2d54cff38dba103cd70462cc778b/specification/2.0#default-material
        let material = prim.material().index()
            .map(|id| materials[id].clone())
            .unwrap_or_default();

        // Not handling optional normals yet
        assert_eq!(positions.len(), normals.len(),
            "glTF geometry must have exactly as many normals as vertices");

        match (&joint_influences, &joint_weights) {
            (Some(joint_influences), Some(joint_weights)) => {
                assert_eq!(positions.len(), joint_influences.len(),
                    "glTF geometry must have exactly as many vertex joint influences as vertices");
                assert_eq!(positions.len(), joint_weights.len(),
                    "glTF geometry must have exactly as many vertex weights as vertices");
            },

            (Some(_), None) | (None, Some(_)) => {
                unreachable!("glTF geometry must always either have both vertex joint influences and vertex weights, or neither");
            },

            (None, None) => {},
        }

        Self {name, indices, positions, normals, tex_coords, joint_influences, joint_weights, material}
    }
}
