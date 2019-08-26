use crate::model::{Material, Scene};
use std::path::Path;
use std::sync::Arc;

/// Loads the given glTF file path, optionally generating a scene for the
/// specific animation or frame within the file.
pub fn load_file(path: impl AsRef<Path>) -> Result<Scene, gltf::Error> {
    GltfFile::load_file(path).map(|file| file.model())
}

#[derive(Debug, Clone)]
pub struct GltfFile {
    scene: Scene,
}

impl GltfFile {
    /// Reads partial glTF data from disk and parses it.
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, gltf::Error> {
        let (document, buffers, _) = gltf::import(path)?;
        let materials: Vec<_> = document
            .materials()
            .map(|material| Arc::new(Material::from(material)))
            .collect();

        Ok(Self {
            scene: Scene::from_gltf(&document, &buffers, &materials),
        })
    }

    /// Returns the default model (all the meshes) for this glTF file
    pub fn model(&self) -> Scene {
        self.scene.clone()
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
    pub fn frame(&self, animation: Option<&str>, frame: Option<usize>) -> Scene {
        self.scene.clone()
        // TODO: write an "animation data" that can be applied to the scene by
        // the renderer to render the correct frame
        /*
        // Create nodes that we can apply transformations to
        let mut nodes = self.document.nodes()
            .map(|gltf_node| Node::from(gltf_node))
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
                        let [x, y, z, w] = iter.next().unwrap();
                        nodes[node_index].apply_rotation(Quaternion::from_xyzw(x, y, z, w));
                    }
                    ReadOutputs::Translations(mut translations) => {
                        nodes[node_index].apply_translation(
                            Vec3::from(translations.next().unwrap())
                        )
                    }
                    ReadOutputs::Scales(mut scales) => {
                        nodes[node_index].apply_scale(
                            Vec3::from(scales.next().unwrap())
                        )
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
        */
    }

    /// Returns the frame index of the last frame in the given animation. If animation is None,
    /// the file must have only a single animation.
    pub fn end_frame(&self, animation: Option<&str>) -> usize {
        //TODO: Use the `animation` parameter when we support glTF animations.

        0
    }
}
