use crate::math::Mat4;

#[derive(Debug, Clone)]
pub struct Camera {
    /// The view matrix of this camera
    pub view: Mat4,
    /// The projection matrix of this camera
    pub projection: Mat4,
}
