use std::path::PathBuf;
use std::num::NonZeroUsize;
use std::f32::consts::PI;

use vek::{Vec3, Mat4};
use serde::{Serialize, Deserialize};

use crate::camera::Camera;

/// A configuration that represents the tasks that spritec should complete
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskConfig {
    /// A list of the spritesheets for spritec to generate
    pub spritesheets: Vec<Spritesheet>,
    /// A list of individual poses for spritec to generate images for
    pub poses: Vec<Pose>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spritesheet {
    /// The path to output the generated spritesheet, relative to configuration file
    pub path: PathBuf,
    /// Animations to include in the spritesheet
    pub animations: Vec<Animation>,
    /// A scale factor to apply to the generated images. Each image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    #[serde(default = "default_scale_factor")]
    pub scale: NonZeroUsize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Animation {
    pub frames: AnimationFrames,
    /// The width at which to render each frame
    pub frame_width: NonZeroUsize,
    /// The height at which to render each frame
    pub frame_height: NonZeroUsize,
    /// The camera perspective from which to render each frame
    pub camera: PresetCamera,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Pose {
    /// The model to render
    pub model: PoseModel,
    /// The path to output the generated image, relative to configuration file
    pub path: PathBuf,
    /// The width at which to render each frame
    pub width: NonZeroUsize,
    /// The height at which to render each frame
    pub height: NonZeroUsize,
    /// The camera perspective from which to render each frame
    pub camera: PresetCamera,
    /// A scale factor to apply to the generated image. The image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    #[serde(default = "default_scale_factor")]
    pub scale: NonZeroUsize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

/// A number of present camera angles or a completely custom configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum PresetCamera {
    Perspective(Perspective),
    Custom {
        /// The position of the camera (world coordinates)
        /// Specify as an object: {x = 1.23, y = 4.56, z = 7.89}
        position: Vec3<f32>,
        /// The target position to look at (world coordinates)
        /// Specify as an object: {x = 1.23, y = 4.56, z = 7.89}
        target: Vec3<f32>,
    },
}

impl From<PresetCamera> for Camera {
    fn from(cam: PresetCamera) -> Self {
        use PresetCamera::*;
        match cam {
            Perspective(persp) => persp.into(),
            //TODO(#4): This should be implemented as part of #4.
            Custom {position, target} => unimplemented!(),
        }
    }
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

impl From<Perspective> for Camera {
    fn from(persp: Perspective) -> Self {
        //TODO(#4): This should be reimplemented properly as part of #4. These placeholder values
        // are only meant to work for the desired angles of bigboi. The angles are slightly tilted.
        // In the actual implementation they should be straight on.
        use Perspective::*;
        let view = match persp {
            PerspectiveFront => Mat4::rotation_x(PI/8.0) * Mat4::rotation_y(0.0*PI/2.0),
            PerspectiveBack => Mat4::rotation_x(PI/8.0) * Mat4::rotation_y(-1.0*PI/2.0),
            PerspectiveLeft => unimplemented!("TODO"),
            PerspectiveRight => unimplemented!("TODO"),
            PerspectiveTop => unimplemented!("TODO"),
            PerspectiveBottom => unimplemented!("TODO"),
        };

        //TODO(#4): This should be implemented as part of #4. We may want to add some additional
        // settings to the Custom variant of PresetCamera. The variables below are good examples of
        // what these additional fields could be called.
        let fov = 0.8*PI; // radians
        let aspect_ratio_x = 1.0;
        let aspect_ratio_y = 1.0;
        let near = 0.01;
        let far = 100.0;
        //TODO(#4): There are several methods with "perspective" in the name for Mat4. Don't know
        // which one we want to use.
        let projection = Mat4::perspective_rh_no(fov, aspect_ratio_x/aspect_ratio_y, near, far)
            //TODO(#4): Part of #4 is that we want to get rid of the scaling here
            * Mat4::<f32>::scaling_3d(0.6);

        Camera {view, projection}
    }
}

fn default_scale_factor() -> NonZeroUsize { NonZeroUsize::new(1).unwrap() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bigboi_config() {
        let conf_str = include_str!("../samples/bigboi/spritec.toml");
        let _: TaskConfig = toml::from_str(conf_str).unwrap();
    }
}
