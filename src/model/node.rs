use crate::model::Material;
use crate::model::Mesh;
use std::collections::HashMap;
use std::sync::Arc;
use vek::{Mat4, Quaternion, Vec3};

/// A node in the scene. A node has a local transform and may contain geometry.
#[derive(Debug)]
pub struct Node {
    meshes: Option<Vec<Mesh>>,
    children: Vec<Arc<Node>>,
    translation: Vec3<f32>,
    rotation: Quaternion<f32>,
    scale: Vec3<f32>,
}

impl Node {
    /// Creates a new node from a vector of meshes.
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Node {
            meshes: Some(meshes),
            children: Vec::<Arc<Node>>::new(),
            translation: Vec3::zero(),
            rotation: Quaternion::identity(),
            scale: Vec3::one(),
        }
    }

    /// Creates a new node from a glTF node. Children nodes that are referenced
    /// by `gltf_node` are also created and stored in `node_map`.
    pub fn from_gltf_node(
        buffers: &[gltf::buffer::Data],
        gltf_node: &gltf::Node,
        node_map: &HashMap<usize, Arc<Node>>,
        materials: &[Arc<Material>]
    ) -> Self {
        let meshes = if let Some(mesh) = gltf_node.mesh() {
            Some(mesh.primitives().map(|primitive| {
                Mesh::from_gltf(
                    buffers,
                    &primitive,
                    materials,
                    Mat4::<f32>::identity()
                )
            }).collect())
        } else {
            None
        };

        let children: Vec<Arc<Node>> =
            gltf_node.children().map(|child_gltf_node| {
                match node_map.get(&child_gltf_node.index()) {
                    Some(child_node) => child_node.clone(),
                    None => Arc::new(Node::from_gltf_node(
                        buffers,
                        &child_gltf_node,
                        node_map,
                        materials,
                    )),
                }
            }).collect();

        Node {
            meshes,
            children,
            translation: Vec3::zero(),
            rotation: Quaternion::identity(),
            scale: Vec3::one(),
        }
    }

    pub fn meshes(&self) -> &Option<Vec<Mesh>> {
        &self.meshes
    }

    pub fn children(&self) -> &[Arc<Node>] {
        self.children.as_slice()
    }

    fn apply_rotation(&mut self, rotation: Quaternion<f32>) {
        self.rotation = rotation;
    }

    fn apply_translation(&mut self, translation: Vec3<f32>) {
        self.translation = translation;
    }

    fn apply_scale(&mut self, scale: Vec3<f32>) {
        self.scale = scale;
    }

    fn local_transform(&self) -> Mat4<f32> {
        Node::apply_transform(
            Mat4::<f32>::identity(),
            self.translation,
            self.rotation,
            self.scale,
        )
    }

    fn apply_transform(
        m: Mat4<f32>,
        translation: Vec3<f32>,
        rotation: Quaternion<f32>,
        scale: Vec3<f32>,
    ) -> Mat4<f32> {
        let mut transform = m;

        // Order of operation: t * r * s, so apply s->r->t
        transform.scale_3d(scale);
        let r_matrix = Mat4::<f32>::from(rotation);
        transform = r_matrix * transform;
        transform.translate_3d(translation);

        transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_transform() {
        let expected = Mat4::<f32>::new(
            -8.0, 0.0, 0.0, 2.0,
            0.0, -10.0, 0.0, 4.0,
            0.0, 0.0, 12.0, 6.0,
            0.0, 0.0, 0.0, 2.0,
        );

        let actual = Node::apply_transform(
            Mat4::from_row_arrays([
                [2.0, 0.0, 0.0, 0.0],
                [0.0, 2.0, 0.0, 0.0],
                [0.0, 0.0, 2.0, 0.0],
                [0.0, 0.0, 0.0, 2.0],
            ]),
            Vec3::new(1.0, 2.0, 3.0),
            // rotate 180 degrees around z-axis
            Quaternion::from_xyzw(0.0, 0.0, 1.0, 0.0),
            Vec3::new(4.0, 5.0, 6.0),
        );
        assert_eq!(expected, actual);
    }
}
