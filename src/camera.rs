use vek::Mat4;

#[derive(Debug, Clone)]
pub struct Camera {
    //TODO(#4): These fields should be replaced with: position, target, fov, aspect_ratio_x, etc.
    pub(crate) view: Mat4<f32>,
    pub(crate) projection: Mat4<f32>,
}

impl Camera {
    /// The view transformation represents the position and orientation of the camera.
    ///
    /// World coordinates -> Camera coordinates
    pub fn view(&self) -> Mat4<f32> {
        self.view
    }

    /// The perspective/orthographic projection of the camera.
    ///
    /// Camera coordinates -> Homogenous coordinates
    pub fn projection(&self) -> Mat4<f32> {
        self.projection
    }
}
