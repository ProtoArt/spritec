use std::f32::consts::PI;
use vek::Mat4;
use vek::Vec4;
use serde::{Serialize, Deserialize};
use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Camera {
     eye: Vec4<f32>,
     target: Vec4<f32>,
     fovy: f32,
     aspect_ratio_x: f32,
     aspect_ratio_y: f32,
     near: f32,
     far: f32,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            eye: Vec4{x:0.0, y:1.0, z:8.5, w:1.0},
            target: Vec4{x:0.0, y:0.0, z:0.0, w:1.0},
            fovy: 0.35*PI,
            aspect_ratio_x: 1.0,
            aspect_ratio_y: 1.0,
            near: 0.01,
            far: 100.0,
        }
    }
}

impl From<config::PresetCamera> for Camera {
    fn from(cam: config::PresetCamera) -> Self {
        use config::PresetCamera::*;
        match cam {
            Perspective(persp) => persp.into(),
            Custom(camera) => camera,
        }
    }
}

impl From<config::Perspective> for Camera {
    fn from(persp: config::Perspective) -> Self {

        // NOTE: PerspectiveLeft means point the camera to the left side of the model
        use config::Perspective::*;
        let eye = match persp {
            PerspectiveFront => Vec4{x:0.0, y:0.0, z:8.5, w:1.0},
            PerspectiveBack => Vec4{x:0.0, y:0.0, z:-8.5, w:1.0},
            PerspectiveLeft => Vec4{x:-8.5, y:0.0, z:0.0, w:1.0},
            PerspectiveRight => Vec4{x:8.5, y:0.0, z:0.0, w:1.0},
            PerspectiveTop => Vec4{x:0.0, y:8.5, z:-1.0, w:1.0},
            PerspectiveBottom => Vec4{x:0.0, y:-8.5, z:-1.0, w:1.0},
        };
        Camera {eye, ..Default::default()}
    }
}

impl Camera {
    /// The view transformation represents the position and orientation of the camera.
    ///
    /// World coordinates -> Camera coordinates
    pub fn view(&self) -> Mat4<f32> {
        Mat4::<f32>::look_at_rh(self.eye, self.target, Vec4::up())
    }

    /// The perspective/orthographic projection of the camera.
    ///
    /// Camera coordinates -> Homogenous coordinates
    pub fn projection(&self) -> Mat4<f32> {
        // OpenGL clip planes match to -1 to 1 and that should work fine for most of our incoming models for now
        // Thus, we use the rh_no(negative one to one) perspective mapping.
        Mat4::perspective_rh_no(self.fovy, self.aspect_ratio_x/self.aspect_ratio_y, self.near, self.far)
    }
}
