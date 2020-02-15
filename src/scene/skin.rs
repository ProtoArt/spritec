use std::iter;

use crate::math::Mat4;

use super::{NodeId};

#[derive(Debug, Clone)]
pub struct Joint {
    /// The node that this joint refers to
    node_id: NodeId,
    /// Matrix that transforms coordinates being skinned into the same space as the joint
    inverse_bind_matrix: Mat4,
}

#[derive(Debug, Clone)]
pub struct Skin {
    joints: Vec<Joint>,
}

impl Skin {
    pub fn from_gltf(
        skin: gltf::Skin,
        buffers: &[gltf::buffer::Data],
    ) -> Self {
        let joints = skin.joints().map(|node| NodeId::from_gltf(&node));

        let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
        let joints = match reader.read_inverse_bind_matrices() {
            Some(ivbm) => joints.zip(ivbm.map(Mat4::from_col_arrays))
                .map(|(node_id, inverse_bind_matrix)| Joint {node_id, inverse_bind_matrix})
                .collect(),

            // From the docs for Skin::inverse_bind_matrices:
            // When None, each matrix is assumed to be the 4x4 identity matrix which implies that
            // the inverse-bind matrices were pre-applied.
            None => joints.zip(iter::repeat(Mat4::identity()))
                .map(|(node_id, inverse_bind_matrix)| Joint {node_id, inverse_bind_matrix})
                .collect(),
        };

        Self {joints}
    }
}
