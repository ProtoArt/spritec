use std::sync::Arc;

use crate::math::{Mat4, Quaternion};

use super::{NodeId, Mesh, Skin, CameraType, LightType};

#[derive(Debug, Clone)]
pub enum NodeData {
    Mesh(Arc<Mesh>, Option<Arc<Skin>>),
    Camera(Arc<CameraType>),
    Light(Arc<LightType>),
}

#[derive(Debug, Clone)]
pub struct Node {
    /// The unique ID of this node. No other node has this ID.
    pub id: NodeId,
    /// The name of the node (possibly empty), or None if the 3D file this was loaded from does
    /// not support node names
    pub name: Option<String>,
    /// The data contained in the node, or None if no data is present
    pub data: Option<NodeData>,
    /// The **local** transform of this node, independent of its parents
    pub transform: Mat4,
}

impl Node {
    pub fn from_gltf(
        node: gltf::Node,
        meshes: &[Arc<Mesh>],
        skins: &[Arc<Skin>],
        cameras: &[Arc<CameraType>],
        lights: &[Arc<LightType>],
    ) -> Self {
        let id = NodeId::from_gltf(&node);
        let name = Some(node.name().unwrap_or("").to_string());

        let data = match (node.mesh(), node.skin(), node.camera(), node.light()) {
            (None, None, None, None) => {
                None
            },

            (Some(mesh), skin, None, None) => {
                let skin = skin.map(|skin| skins[skin.index()].clone());
                Some(NodeData::Mesh(meshes[mesh.index()].clone(), skin))
            },

            (None, None, Some(cam), None) => {
                Some(NodeData::Camera(cameras[cam.index()].clone()))
            },

            (None, None, None, Some(light)) => {
                Some(NodeData::Light(lights[light.index()].clone()))
            },

            (_, Some(_), _, _) => unreachable!("Did not expect a node that had a skin but no mesh"),
            _ => unreachable!("Did not expect a node that had more than one of a mesh, camera, or light"),
        };

        use gltf::scene::Transform::*;
        let transform = match node.transform() {
            Matrix {matrix} => Mat4::from_col_arrays(matrix),
            Decomposed {translation, rotation: [rx, ry, rz, rw], scale} => {
                let scale_mat = Mat4::scaling_3d(scale);
                // This code assumes that glTF provides us with **normalized** quaternions
                let rot_mat = Mat4::from(Quaternion::from_xyzw(rx, ry, rz, rw));
                let trans_mat = Mat4::translation_3d(translation);

                // glTF allows us to construct a matrix by performing T * R * S
                // See: https://github.com/KhronosGroup/glTF/tree/master/specification/2.0#transformations
                trans_mat * rot_mat * scale_mat
            },
        };

        Self {id, name, data, transform}
    }

    pub fn mesh(&self) -> Option<(&Arc<Mesh>, Option<&Arc<Skin>>)> {
        match &self.data {
            Some(NodeData::Mesh(mesh, skin)) => Some((mesh, skin.as_ref())),
            _ => None,
        }
    }

    pub fn camera(&self) -> Option<&Arc<CameraType>> {
        match &self.data {
            Some(NodeData::Camera(cam)) => Some(cam),
            _ => None,
        }
    }

    pub fn light(&self) -> Option<&Arc<LightType>> {
        match &self.data {
            Some(NodeData::Light(light)) => Some(light),
            _ => None,
        }
    }
}
