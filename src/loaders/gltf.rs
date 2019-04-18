use std::sync::Arc;
use std::path::Path;

use crate::model::{Mesh, Material, Model};

/// Loads the given glTF file path, optionally generating a model for the specific animation or
/// frame within the file.
pub fn load_file(path: impl AsRef<Path>) -> Result<Model, gltf::Error> {
    GltfFile::load_file(path).map(|file| file.model())
}

#[derive(Debug, Clone)]
pub struct GltfFile {
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
}

impl GltfFile {
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, gltf::Error> {
        let (document, buffers, _) = gltf::import(path)?;

        Ok(Self {document, buffers})
    }

    /// Returns the default model (all the meshes) for this glTF file
    pub fn model(&self) -> Model {
        // Load all the materials first, this assumes that the material index
        // that primitive refers to is loaded in the same order as document.materials()
        let materials: Vec<_> = self.document.materials()
            .map(|material| Arc::new(Material::from(material)))
            .collect();

        let mut meshes = Vec::new();
        for mesh in self.document.meshes() {
            for primitive in mesh.primitives() {
                meshes.push(Mesh::from_gltf(&self.buffers, &primitive, &materials));
            }
        }

        Model {meshes}
    }

    /// Return the particular frame of the given animation
    ///
    /// The `animation` parameter is the name of the animation to select. Can be omitted if there
    /// is only a single animation or if there is no animation.
    ///
    /// The `frame` parameter is the specific animation frame to render. The default is to render
    /// the first frame (or the loaded pose of the model if there is no animation)
    pub fn frame(&self, animation: Option<&str>, frame: Option<usize>) -> Model {
        //TODO: Use the `animation` and `frame` parameter when we support glTF animations. This
        // will probably involve refactoring this struct considerably (the interface should stay
        // stay the same though).

        self.model()
    }

    /// Returns the frame index of the last frame in the given animation. If animation is None,
    /// the file must have only a single animation.
    pub fn end_frame(&self, animation: Option<&str>) -> usize {
        //TODO: Use the `animation` parameter when we support glTF animations.

        0
    }
}
