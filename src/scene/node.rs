use std::sync::Arc;
use std::collections::VecDeque;

use crate::math::{Mat4, Quaternion};

use super::{Mesh, CameraType, LightType};

#[derive(Debug, Clone)]
pub enum NodeData {
    Mesh(Arc<Mesh>),
    Camera(Arc<CameraType>),
    Light(Arc<LightType>),
}

#[derive(Debug, Clone)]
pub struct Node {
    /// The data contained in the node, or None if no data is present
    pub data: Option<NodeData>,
    /// The **local** transform of this node, independent of its parents
    pub transform: Mat4,
    /// The children of this node
    ///
    /// Each child's global transform is dependent on this node's transform
    pub children: Vec<Arc<Node>>,
}

impl Node {
    pub fn from_gltf(
        node: gltf::Node,
        meshes: &[Arc<Mesh>],
        cameras: &[Arc<CameraType>],
        lights: &[Arc<LightType>],
    ) -> Self {
        let data = match (node.mesh(), node.camera(), node.light()) {
            (None, None, None) => {
                None
            },

            (Some(mesh), None, None) => {
                Some(NodeData::Mesh(meshes[mesh.index()].clone()))
            },

            (None, Some(cam), None) => {
                Some(NodeData::Camera(cameras[cam.index()].clone()))
            },

            (None, None, Some(light)) => {
                Some(NodeData::Light(lights[light.index()].clone()))
            },

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

        // Important property: Every unique node in the scene graph is represented by a single
        // Arc<Node>. That is, we are careful to never call from_gltf on the same node twice.
        //
        // From the glTF spec:
        // > For Version 2.0 conformance, the glTF node hierarchy is not a directed acyclic graph
        // > (DAG) or scene graph, but a disjoint union of strict trees. That is, no node may be a
        // > direct descendant of more than one node. This restriction is meant to simplify
        // > implementation and facilitate conformance.
        //
        // This code only works because the node hierarchy is a tree. Otherwise, it would recurse
        // forever and we'd have to rewrite it to use two passes and cycle detection.
        let children = node.children()
            .map(|child| Arc::new(Node::from_gltf(child, meshes, cameras, lights)))
            .collect();

        Self {data, transform, children}
    }

    pub fn mesh(&self) -> Option<&Arc<Mesh>> {
        match &self.data {
            Some(NodeData::Mesh(mesh)) => Some(mesh),
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

// An extension trait for Arc<Node> that provides a way to traverse the nodes
// This needs to be a trait because we can't add methods to Arc<Node> directly
pub trait Traverse {
    /// Traverse a node hierarchy, treating self as a root node, yielding each node and the world
    /// transform of that node's parent. Note that since this is a world transform, it will reflect
    /// the total transformation up the entire hierarchy.
    fn traverse(&self) -> TraverseNodes;
}

impl Traverse for Arc<Node> {
    fn traverse(&self) -> TraverseNodes {
        let mut queue = VecDeque::new();
        queue.push_back((Mat4::identity(), self.clone()));
        TraverseNodes {queue}
    }
}

pub struct TraverseNodes {
    /// A queue of each node to be traversed, and its parent transform
    queue: VecDeque<(Mat4, Arc<Node>)>,
}

impl Iterator for TraverseNodes {
    type Item = (Mat4, Arc<Node>);

    fn next(&mut self) -> Option<Self::Item> {
        // This code assumes that the node hierarchy is not cyclic
        let (parent_trans, node) = self.queue.pop_front()?;

        let world_transform = parent_trans * node.transform;
        self.queue.extend(node.children.iter().map(|node| (world_transform, node.clone())));

        Some((parent_trans, node))
    }
}
