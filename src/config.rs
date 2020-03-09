use std::num::NonZeroU32;

use serde::{Serialize, Deserialize};

use crate::math::{Vec3, Rgba, Degrees, Milliseconds};

// PathBuf is not imported to avoid its use in this module. Every path in this module should
// be an UnresolvedPath.
use std::path::{Path, Component};

/// A newtype around PathBuf to force the path to be resolved relative to a base directory before
/// it can be used. Good to prevent something that is pretty easy to do accidentally.
// Using an absolute path to PathBuf so we don't even have PathBuf imported
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnresolvedPath(std::path::PathBuf);

impl UnresolvedPath {
    /// Resolves this path relative to the given base directory. Returns an absolute path.
    pub fn resolve(&self, base_dir: &Path) -> std::path::PathBuf {
        use std::path::PathBuf;

        // Path resolution based on code found at
        // https://github.com/rust-lang/cargo/blob/9ef364a5507ef87843c5f37b11d3ccfbd8cbe478/src/cargo/util/paths.rs#L65-L90
        //
        // Resolution removes . and .. from the path, where . is removed without affecting the rest
        // of the path and .. will remove its parent from the path.
        // This is needed because Windows extended-length paths disallow string parsing, so ".."
        // and "." aren't resolved in path names.
        //
        // Reference: https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file

        // Constraint: The base directory path (base_dir) should always be an absolute path.
        assert!(base_dir.is_absolute(), "Base directory path was not absolute.");

        let UnresolvedPath(path) = self;
        // Create an absolute path, using base_dir only if necessary
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            base_dir.join(path)
        };

        // All slashes are converted to backslahes (for Windows) or to forward slashes (for every other OS)
        // so that PathBuf's components function can properly extract out the components of the path
        #[cfg(windows)]
        let path_str = path.to_str()
            .expect("Path was not valid Unicode")
            .replace("/", "\\");
        #[cfg(not(windows))]
        let path_str = path.to_str()
            .expect("Path was not valid Unicode")
            .replace("\\", "/");
        let path = Path::new(&path_str);

        let mut normalized_path = PathBuf::new();
        for component in path.components().peekable() {
            match component {
                Component::Prefix(_) |
                Component::RootDir => normalized_path.push(component.as_os_str()),
                Component::CurDir => {},
                Component::ParentDir => {
                    normalized_path.pop();
                },
                Component::Normal(c) => normalized_path.push(c),
            }
        }

        normalized_path
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
    /// The width of each cell in the spritesheet in pixels
    pub cell_width: NonZeroU32,
    /// The height of each cell in the spritesheet in pixels
    pub cell_height: NonZeroU32,
    /// Animations to include in the spritesheet
    pub animations: Vec<Animation>,
    /// A scale factor to apply to the generated images. Each image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    #[serde(default = "default_scale_factor")]
    pub scale: NonZeroU32,
    /// The background color of the spritesheet (default: transparent black)
    #[serde(default = "default_background")]
    pub background: Rgba,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Animation {
    pub frames: AnimationFrames,
    /// The width at which to render each frame (in pixels)
    pub frame_width: NonZeroU32,
    /// The height at which to render each frame (in pixels)
    pub frame_height: NonZeroU32,
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
        /// The "global" animation time in ms at which to start the animation (default: 0.0)
        #[serde(default)]
        start_time: Milliseconds,
        /// The "global" animation time in ms at which to end the animation (default: time of the
        /// last keyframe in the animation)
        end_time: Option<Milliseconds>,
        /// The number of steps to take between the start and end time.
        ///
        /// This is the number of frames that will be drawn in the spritesheet.
        steps: NonZeroU32,
    },
    /// An array of filenames. OBJ files will be used as is. For glTF files, the scene will be used
    /// as loaded regardless of the animations present in the file.
    Models(Vec<UnresolvedPath>),
}

impl AnimationFrames {
    /// Returns the number of frame images to be created
    pub fn len(&self) -> u32 {
        use AnimationFrames::*;
        match self {
            GltfFrames {steps, ..} => steps.get(),
            Models(models) => models.len() as u32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pose {
    /// The model to render
    pub model: PoseModel,
    /// The path to output the generated image, relative to configuration file
    pub path: UnresolvedPath,
    /// The width at which to render each frame (in pixels)
    pub width: NonZeroU32,
    /// The height at which to render each frame (in pixels)
    pub height: NonZeroU32,
    /// The camera perspective from which to render each frame
    pub camera: PresetCamera,
    /// A scale factor to apply to the generated image. The image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    #[serde(default = "default_scale_factor")]
    pub scale: NonZeroU32,
    /// The background color of the generated image (default: transparent black)
    #[serde(default = "default_background")]
    pub background: Rgba,
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
        /// The specific time in ms in the animation to render. The default is to render the start
        /// of the animation, or the default pose of the model if there is no animation.
        #[serde(default)]
        time: Milliseconds,
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
    pub color: Rgba,
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
    Named(NamedCamera),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedCamera {
    /// The name of the camera in the 3D model file
    pub name: String,
    /// The name of the scene to look for the camera in or None if the default scene should be used
    pub scene: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Camera {
    /// The position of the camera in world coordinates
    pub eye: Vec3,
    /// The target position that the camera should be looking at
    pub target: Vec3,
    /// The aspect ratio of the viewport
    pub aspect_ratio: f32,
    /// Field of view angle in the y-direction - the "opening angle" of the camera in degrees
    pub fov_y: Degrees,
    /// Coordinate of the near clipping plane on the camera's local z-axis
    pub near_z: f32,
    /// Coordinate of the far clipping plane on the camera's local z-axis
    ///
    /// If None, a special "infinite projection matrix" will be used.
    pub far_z: Option<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Vec3 {x: 8.0, y: 8.0, z: 8.0},
            target: Vec3::zero(),
            aspect_ratio: 1.0,
            fov_y: Degrees::from_degrees(40.0),
            near_z: 0.1,
            far_z: Some(100.0),
        }
    }
}

impl From<Perspective> for Camera {
    fn from(persp: Perspective) -> Self {

        // NOTE: PerspectiveLeft means point the camera to the left side of the model
        use Perspective::*;
        let eye = match persp {
            PerspectiveFront => Vec3 {x: 0.0, y: 0.0, z: 8.5},
            PerspectiveBack => Vec3 {x: 0.0, y: 0.0, z: -8.5},
            PerspectiveLeft => Vec3 {x: -8.5, y: 0.0, z: 0.0},
            PerspectiveRight => Vec3 {x: 8.5, y: 0.0, z: 0.0},
            PerspectiveTop => Vec3 {x: 0.0, y: 8.5, z: -1.0},
            PerspectiveBottom => Vec3 {x: 0.0, y: -8.5, z: -1.0},
        };
        Camera {eye, ..Default::default()}
    }
}

fn default_scale_factor() -> NonZeroU32 { NonZeroU32::new(1).unwrap() }
fn default_background() -> Rgba { Rgba {r: 0.0, g: 0.0, b: 0.0, a: 0.0} }

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! resolve_check {
        ($base:expr, $input:expr, $output:expr) => {
            let input_path = UnresolvedPath(std::path::PathBuf::from($input));
            let base_path = Path::new($base);
            assert_eq!(input_path.resolve(base_path), std::path::PathBuf::from($output));
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn resolve_absolute_path() {
        let input_path = "/home/yourname/spritec/sample/bigboi/test";

        let base_path = "/home/yourname/spritec/sample";
        let output_path = "/home/yourname/spritec/sample/bigboi/test";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_absolute_path() {
        let input_path = "C:\\user\\yourname\\spritec\\sample\\bigboi\\test";

        let base_path = "C:\\user\\yourname\\spritec\\sample";
        let output_path = "C:\\user\\yourname\\spritec\\sample\\bigboi\\test";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(not(windows))]
    #[test]
    fn resolve_relative_path_forward_slash() {
        let input_path = "../../../spritec/../src/./bin";

        let base_path = "/home/yourname/spritec";
        let output_path = "/src/bin";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_relative_path_forward_slash() {
        let input_path = "../../../spritec/../src/./bin";

        let base_path = "C:\\user\\yourname\\spritec";
        let output_path = "C:\\src\\bin";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(not(windows))]
    #[test]
    fn resolve_relative_path_backslash() {
        let input_path = "..\\..\\..\\spritec\\..\\src\\.\\bin";

        let base_path = "/home/yourname/spritec";
        let output_path = "/src/bin";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_relative_path_backslash() {
        let input_path = "..\\..\\..\\spritec\\..\\src\\.\\bin";

        let base_path = "C:\\user\\yourname\\spritec";
        let output_path = "C:\\src\\bin";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_relative_path_backslash_prefix() {
        let input_path = "..\\..\\..\\spritec\\..\\src\\.\\bin";

        // This special prefix is an old Windows feature that no one uses
        let base_path = "\\\\?\\C:\\user\\yourname\\spritec";
        let output_path = "\\\\?\\C:\\src\\bin";
        resolve_check!(base_path, input_path, output_path);
    }

    #[test]
    fn parse_bigboi_config() {
        let conf_str = include_str!("../samples/bigboi/spritec.toml");
        let _: TaskConfig = toml::from_str(conf_str).unwrap();
    }
}
