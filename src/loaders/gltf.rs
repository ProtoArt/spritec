use std::path::Path;
use std::sync::Arc;

use vek::{Vec3, Mat4, Quaternion};

use crate::model::{Material, Mesh, Model};

/// Loads the given glTF file path, optionally generating a model for the specific animation or
/// frame within the file.
pub fn load_file(path: impl AsRef<Path>) -> Result<Model, gltf::Error> {
    GltfFile::load_file(path).map(|file| file.model())
}

#[derive(Debug, Clone)]
pub struct GltfFile {
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    materials: Vec<Arc<Material>>,
}

/// glTF node that can be transformed
struct Node<'a> {
    gltf_node: gltf::Node<'a>,
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}

impl<'a> Node<'a> {
    fn new(gltf_node: gltf::Node<'a>) -> Self {
        Node {
            gltf_node,
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    fn apply_rotation(&mut self, rotation: [f32; 4]) {
        self.rotation = rotation;
    }

    fn apply_translation(&mut self, translation: [f32; 3]) {
        self.translation = translation;
    }

    fn apply_scale(&mut self, scale: [f32; 3]) {
        self.scale = scale;
    }

    fn local_transform(&self) -> Mat4<f32> {
        Node::apply_transform(
            self.gltf_node.transform().matrix(),
            Vec3::<f32>::from(self.translation),
            Quaternion::<f32>::from_xyzw(
                self.rotation[0], self.rotation[1], self.rotation[2], self.rotation[3]
            ),
            Vec3::<f32>::from(self.scale),
        )
    }

    fn apply_transform(
        m: [[f32; 4]; 4],
        translation: Vec3<f32>,
        rotation: Quaternion<f32>,
        scale: Vec3<f32>
    ) -> Mat4<f32> {
        let mut transform = Mat4::<f32>::new(
            m[0][0], m[0][1], m[0][2], m[0][3],
            m[1][0], m[1][1], m[1][2], m[1][3],
            m[2][0], m[2][1], m[2][2], m[2][3],
            m[3][0], m[3][1], m[3][2], m[3][3],
        );

        // Order of operation: t * r * s, so apply s->r->t
        transform.scale_3d(scale);
        let r_matrix = Mat4::<f32>::from(rotation);
        transform = r_matrix * transform;
        transform.translate_3d(translation);

        transform
    }
}

impl GltfFile {
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, gltf::Error> {
        let (document, buffers, _) = gltf::import(path)?;

        // Load all the materials first, this assumes that the material index
        // that primitive refers to is loaded in the same order as document.materials()
        let materials: Vec<_> = document
            .materials()
            .map(|material| Arc::new(Material::from(material)))
            .collect();

        Ok(Self {document, buffers, materials})
    }

    /// Returns the default model (all the meshes) for this glTF file
    pub fn model(&self) -> Model {
        let mut meshes = Vec::new();
        for mesh in self.document.meshes() {
            for primitive in mesh.primitives() {
                meshes.push(Mesh::from_gltf(
                    &self.buffers,
                    &primitive,
                    &self.materials,
                    Mat4::<f32>::identity(),
                ));
            }
        }

        Model { meshes }
    }

    /// Return the particular frame of the given animation
    ///
    /// The `animation` parameter is the name of the animation to select. Can be omitted if there
    /// is only a single animation or if there is no animation.
    ///
    /// The `frame` parameter is the specific animation frame to render. The default is to render
    /// the first frame (or the loaded pose of the model if there is no animation)
    ///
    /// TODO: need to be given frame_rate in order to know which frame to render
    pub fn frame(&self, animation: Option<&str>, frame: Option<usize>) -> Model {
        // Create nodes that we can apply transformations to
        let mut nodes = self.document.nodes()
            .map(|gltf_node| Node::new(gltf_node))
            .collect::<Vec<Node>>();

        // TODO: currently only handling the first animation, need to read animation
        if let Some(animation) = self.document.animations().next() {
            for channel in animation.channels() {
                let reader = channel.reader(|buffer| Some(&self.buffers[buffer.index()]));
                let outputs = reader
                    .read_outputs()
                    .expect("Can read gltf animation sampler output");
                let node_index = channel.target().node().index();

                // TODO: interpolation; currently only applying the first keyframe,
                // keyframe times can be read from `reader.read_inputs()`
                use gltf::animation::util::ReadOutputs;
                match outputs {
                    ReadOutputs::Rotations(rotations) => {
                        let mut iter = rotations.into_f32();
                        nodes[node_index].apply_rotation(iter.next().unwrap());
                    }
                    ReadOutputs::Translations(mut translations) => {
                        nodes[node_index].apply_translation(translations.next().unwrap())
                    }
                    ReadOutputs::Scales(mut scales) => {
                        nodes[node_index].apply_scale(scales.next().unwrap())
                    }
                    ReadOutputs::MorphTargetWeights(_) => {
                        // TODO: gltf morph targets not supported
                        println!("gltf animation morph target weights not supported, ignoring")
                    }
                }
            }

            // TODO: render a scene and walk the transform hierarchy tree to get the global
            //       transform instead of local transform
            // see reference https://github.com/KhronosGroup/glTF-Sample-Viewer/blob/master/src/scene.js
            let mut meshes = Vec::new();
            nodes.iter().for_each(|node| {
                if let Some(mesh) = node.gltf_node.mesh() {
                    for primitive in mesh.primitives() {
                        meshes.push(Mesh::from_gltf(
                            &self.buffers,
                            &primitive,
                            &self.materials,
                            node.local_transform(),
                        ));
                    }
                }
            });

            Model { meshes }
        } else {
            self.model()
        }
    }

    /// Returns the frame index of the last frame in the given animation. If animation is None,
    /// the file must have only a single animation.
    pub fn end_frame(&self, animation: Option<&str>) -> usize {
        //TODO: Use the `animation` parameter when we support glTF animations.

        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_transform() {
        let expected = Mat4::<f32>::new(
            -8.0,   0.0,  0.0, 2.0,
             0.0, -10.0,  0.0, 4.0,
             0.0,   0.0, 12.0, 6.0,
             0.0,   0.0,  0.0, 2.0,
        );

        let actual = Node::apply_transform(
            [
                [2.0, 0.0, 0.0, 0.0],
                [0.0, 2.0, 0.0, 0.0],
                [0.0, 0.0, 2.0, 0.0],
                [0.0, 0.0, 0.0, 2.0]
            ],
            Vec3::new(1.0, 2.0, 3.0),
            // rotate 180 degrees around z-axis
            Quaternion::from_xyzw(0.0, 0.0, 1.0, 0.0),
            Vec3::new(4.0, 5.0, 6.0)
        );
        assert_eq!(expected, actual);
    }
}
