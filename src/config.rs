use std::num::{NonZeroUsize, NonZeroU32};

use vek::{Vec3, Rgba};
use serde::{Serialize, Deserialize};
use crate::shader::Camera;

/// A newtype around PathBuf to force the path to be resolved relative to a base directory before
/// it can be used. Good to prevent something that is pretty easy to do accidentally.
// Using an absolute path to PathBuf so we don't even have PathBuf imported
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnresolvedPath(std::path::PathBuf);

impl UnresolvedPath {
    /// Resolves this path relative to the given base directory. Returns an absolute path.
    pub fn resolve(&self, base_dir: &std::path::Path) -> std::path::PathBuf {
        let path = &self.0;
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            base_dir.join(path)
        }
    }
}

/// A configuration that represents the tasks that spritec should complete
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskConfig {
    /// A list of the spritesheets for spritec to generate
    #[serde(default)]
    pub spritesheets: Vec<Spritesheet>,
    /// A list of individual poses for spritec to generate images for
    #[serde(default)]
    pub poses: Vec<Pose>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spritesheet {
    /// The path to output the generated spritesheet, relative to configuration file
    pub path: UnresolvedPath,
    /// Animations to include in the spritesheet
    pub animations: Vec<Animation>,
    /// A scale factor to apply to the generated images. Each image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    #[serde(default = "default_scale_factor")]
    pub scale: NonZeroU32,
    /// The background color of the spritesheet (default: transparent black)
    #[serde(default = "default_background")]
    pub background: Rgba<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Animation {
    pub frames: AnimationFrames,
    /// The width at which to render each frame (in pixels)
    pub frame_width: NonZeroUsize,
    /// The height at which to render each frame (in pixels)
    pub frame_height: NonZeroUsize,
    /// The camera perspective from which to render each frame
    pub camera: PresetCamera,
    /// The outline to use when drawing each frame. (default: no outline)
    #[serde(default)]
    pub outline: Outline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum AnimationFrames {
    GltfFrames {
        /// The path to a glTF file
        gltf: UnresolvedPath,
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
    Models(Vec<UnresolvedPath>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pose {
    /// The model to render
    pub model: PoseModel,
    /// The path to output the generated image, relative to configuration file
    pub path: UnresolvedPath,
    /// The width at which to render each frame (in pixels)
    pub width: NonZeroUsize,
    /// The height at which to render each frame (in pixels)
    pub height: NonZeroUsize,
    /// The camera perspective from which to render each frame
    pub camera: PresetCamera,
    /// A scale factor to apply to the generated image. The image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    #[serde(default = "default_scale_factor")]
    pub scale: NonZeroU32,
    /// The background color of the generated image (default: transparent black)
    #[serde(default = "default_background")]
    pub background: Rgba<f32>,
    /// The outline to use when drawing the generated image. (default: no outline)
    #[serde(default)]
    pub outline: Outline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum PoseModel {
    GltfFrame {
        /// The path to a glTF file
        gltf: UnresolvedPath,
        /// The name of the animation to select. Can be omitted if there is only a single animation
        /// or if there is no animation.
        animation: Option<String>,
        /// The specific animation frame to render. The default is to render the first frame (or
        /// the loaded pose of the model if there is no animation)
        frame: Option<usize>,
    },
    /// A single filename. An OBJ file will be used as is. For a glTF file, the model will be
    /// rendered as loaded regardless of the animations present in the file.
    Model(UnresolvedPath),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Outline {
    /// The outline thickness to use when drawing the generated image. Value must not be negative.
    /// (default: 0.0)
    pub thickness: f32,
    /// The color of the outline to draw (default: black)
    pub color: Rgba<f32>,
}

impl Default for Outline {
    fn default() -> Self {
        Self {
            thickness: 0.0,
            color: Rgba::black(),
        }
    }
}

/// A number of present camera angles or a completely custom configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum PresetCamera {
    Perspective(Perspective),
    Custom(Camera),
}

/// Preset perspective cameras for common angles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Perspective {
    PerspectiveFront,
    PerspectiveBack,
    PerspectiveLeft,
    PerspectiveRight,
    PerspectiveTop,
    PerspectiveBottom,
}

fn default_scale_factor() -> NonZeroU32 { NonZeroU32::new(1).unwrap() }
fn default_background() -> Rgba<f32> { Rgba {r: 0.0, g: 0.0, b: 0.0, a: 0.0} }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bigboi_config() {
        let conf_str = include_str!("../samples/bigboi/spritec.toml");
        let _: TaskConfig = toml::from_str(conf_str).unwrap();
    }
}
