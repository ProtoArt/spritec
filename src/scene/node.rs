use std::sync::Arc;
use vek::{Mat4, Vec3, Quaternion};

use super::{Mesh, CameraType};

#[derive(Debug, Clone)]
pub enum NodeData {
    Mesh(Arc<Mesh>),
    Camera(Arc<CameraType>),
    //TODO: Lighting support
    //Light(Arc<LightType>),
}

#[derive(Debug, Clone)]
pub struct Node {
    /// The data contained in the node, or None if no data is present
    pub data: Option<NodeData>,
    /// The **local** transform of this node, independent of its parents
    pub transform: Mat4<f32>,
    /// The children of this node
    ///
    /// Each child's global transform is dependent on this node's transform
    pub children: Vec<Arc<Node>>,
}

impl Node {
    pub fn camera(eye: Vec3<f32>, target: Vec3<f32>, camera: Arc<CameraType>) -> Self {
        Self {
            data: Some(NodeData::Camera(camera)),
            transform: Mat4::model_look_at_rh(eye, target, Vec3::up()),
            children: Vec::new(),
        }
    }

    pub fn from_gltf(
        node: gltf::Node,
        meshes: &[Arc<Mesh>],
        cameras: &[Arc<CameraType>],
    ) -> Self {
        //TODO: Add lighting support by calling node.light() in a third field of this tuple
        let data = match (node.mesh(), node.camera()/*, node.light()*/) {
            (None, None/*, None*/) => {
                None
            },

            (Some(mesh), None/*, None*/) => {
                Some(NodeData::Mesh(meshes[mesh.index()].clone()))
            },

            (None, Some(cam)/*, None*/) => {
                Some(NodeData::Camera(cameras[cam.index()].clone()))
            },

            //(None, None, Some(light)) => {
            //    //TODO: Lighting support
            //    unimplemented!()
            //},

            _ => unreachable!("Did not expect a node that had more than one of a mesh, camera, or light"),
        };

        use gltf::scene::Transform::*;
        let transform = match node.transform() {
            Matrix {matrix} => Mat4::from_col_arrays(matrix),
            Decomposed {translation, rotation: [rx, ry, rz, rw], scale} => {
                let scale_mat: Mat4<f32> = Mat4::scaling_3d(scale);
                // This code assumes that glTF provides us with **normalized** quaternions
                let rot_mat = Mat4::from(Quaternion::from_xyzw(rx, ry, rz, rw));
                let trans_mat: Mat4<f32> = Mat4::translation_3d(translation);

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
            .map(|child| Arc::new(Node::from_gltf(child, meshes, cameras)))
            .collect();

        Self {data, transform, children}
    }
}

// An extension trait for Arc<Node> that provides a way to traverse the nodes
// This needs to be a trait because we can't add methods to Arc<Node> directly
trait TraverseNodes {
    /// Traverse a node hierarchy, calling the given closure with each node and the world transform
    /// of that node's parent. Note that since this is a world transform, it will reflect the total
    /// transformation up the entire hierarchy.
    ///
    /// Set `parent_trans` to `Mat4::identity()` when calling this on the root node of a scene.
    fn traverse<F: FnMut(Mat4<f32>, &Self)>(&self, f: F, parent_trans: Mat4<f32>);
}

impl TraverseNodes for Arc<Node> {
    fn traverse<F: FnMut(Mat4<f32>, &Self)>(&self, mut f: F, parent_trans: Mat4<f32>) {
        (&mut f)(parent_trans, self);

        // The world transformation of this node
        let world_trans = parent_trans * self.transform;
        for child in &self.children {
            child.traverse(&mut f, world_trans);
        }
    }
}
