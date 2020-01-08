use vek::Mat4;

#[derive(Debug, Clone)]
pub struct Camera {
    /// The view matrix of this camera
    pub view: Mat4<f32>,
    /// The projection matrix of this camera
    pub projection: Mat4<f32>,
}
