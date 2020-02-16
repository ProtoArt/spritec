use std::iter;

use crate::math::Mat4;

use super::{NodeId, NodeWorldTransforms};

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

    /// Computes the joint matrices using the skin data
    ///
    /// See: https://github.com/KhronosGroup/glTF-Tutorials/blob/89bb8706ec3037a38e5ed1b77b5e6a4c3038db3d/gltfTutorial/gltfTutorial_020_Skins.md#the-joint-matrices
    pub fn joint_matrices<'a>(
        &'a self,
        model_transform: Mat4,
        node_world_transforms: &'a NodeWorldTransforms,
    ) -> impl Iterator<Item=Mat4> + 'a {
        let inverse_model_transform = model_transform.inverted();
        self.joints.iter().map(move |joint| {
            let &Joint {node_id, inverse_bind_matrix} = joint;
            // the world transform of the joint
            let joint_transform = node_world_transforms.get(node_id);

            // From the reference above, the formula is:
            // jointMatrix(j) =
            //   globalTransformOfNodeThatTheMeshIsAttachedTo^-1 *
            //   globalTransformOfJointNode(j) *
            //   inverseBindMatrixForJoint(j);
            inverse_model_transform * joint_transform * inverse_bind_matrix
        })
    }
}
