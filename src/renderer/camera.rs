use vek::Mat4;

use crate::scene::CameraType;

#[derive(Debug, Clone)]
pub struct Camera {
    /// The view matrix of this camera
    pub view: Mat4<f32>,
    /// The projection matrix of this camera
    pub projection: Mat4<f32>,
}

impl Camera {
    /// Creates a new camera from a camera type and the world transform of the camera node
    pub fn new(cam: &CameraType, transform: Mat4<f32>) -> Self {
        Self {
            view: transform.inverted(),
            projection: cam.to_projection(),
        }
    }
}
