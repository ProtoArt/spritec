use std::path::PathBuf;
use std::num::NonZeroUsize;

use vek::Vec3;
use serde::{Serialize, Deserialize};

/// A configuration that represents the tasks that spritec should complete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// A list of the spritesheets for spritec to generate
    spritesheets: Vec<Spritesheet>,
    /// A list of individual poses for spritec to generate images for
    poses: Vec<Pose>,
}

fn default_scale_factor() -> NonZeroUsize { NonZeroUsize::new(1).unwrap() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spritesheet {
    /// The path to output the generated spritesheet, relative to configuration file
    path: PathBuf,
    /// Animations to include in the spritesheet
    animations: Vec<Animation>,
    /// A scale factor to apply to the generated images. Each image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    #[serde(default = "default_scale_factor")]
    scale: NonZeroUsize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    frames: AnimationFrames,
    /// The width at which to render each frame
    frame_width: NonZeroUsize,
    /// The height at which to render each frame
    frame_height: NonZeroUsize,
    /// The camera perspective from which to render each frame
    camera: Camera,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimationFrames {
    GltfFrames {
        /// The path to a glTF file
        gltf: PathBuf,
        /// The name of the animation to select. Can be omitted if there is only a single animation
        animation: Option<String>,
        /// The frame to start the animation from (default: 0)
        #[serde(default)]
        start_frame: usize,
        /// The frame to end the animation at (default: last frame of the animation)
        end_frame: Option<usize>,
    },
    /// An array of filenames. OBJ files will be used as is. For glTF files, the model will be used
    /// as loaded regardless of the animations present in the file.
    Models(Vec<PathBuf>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    model: PoseModel,
    /// The width at which to render each frame
    width: NonZeroUsize,
    /// The height at which to render each frame
    height: NonZeroUsize,
    /// The camera perspective from which to render each frame
    camera: Camera,
    /// A scale factor to apply to the generated image. The image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    #[serde(default = "default_scale_factor")]
    scale: NonZeroUsize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PoseModel {
    GltfFrame {
        /// The path to a glTF file
        gltf: PathBuf,
        /// The name of the animation to select. Can be omitted if there is only a single animation
        /// or if there is no animation.
        animation: Option<String>,
        /// The specific animation frame to render. The default is to render the first frame (or
        /// the loaded pose of the model if there is no animation)
        frame: Option<usize>,
    },
    /// A single filename. An OBJ file will be used as is. For a glTF file, the model will be
    /// rendered as loaded regardless of the animations present in the file.
    Model(PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Camera {
    Perspective(Perspective),
    Arbitrary {
        /// The position of the camera (world coordinates)
        /// Specify as an object: {x = 1.23, y = 4.56, z = 7.89}
        position: Vec3<f32>,
        /// The target position to look at (world coordinates)
        /// Specify as an object: {x = 1.23, y = 4.56, z = 7.89}
        target: Vec3<f32>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Perspective {
    PerspectiveFront,
    PerspectiveBack,
    PerspectiveLeft,
    PerspectiveRight,
    PerspectiveTop,
    PerspectiveBottom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bigboi_config() {
        let conf_str = include_str!("../samples/bigboi/spritec.toml");
        let _: TaskConfig = toml::from_str(conf_str).unwrap();
    }
}
