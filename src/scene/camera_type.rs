use crate::math::{Mat4, FrustumPlanes, Radians};

#[derive(Debug, Clone)]
pub enum CameraType {
    Perspective {
        /// The aspect ratio of the viewport
        aspect_ratio: f32,
        /// Field of view in the y-direction - the "opening angle" of the camera in radians
        field_of_view_y: Radians,
        /// Coordinate of the near clipping plane on the camera's local z-axis
        near_z: f32,
        /// Coordinate of the far clipping plane on the camera's local z-axis
        ///
        /// If None, a special "infinite projection matrix" will be used.
        far_z: Option<f32>,
    },

    Orthographic {
        /// The magnification of the camera in the x-direction
        ///
        /// Basically the width of the viewing volume
        mag_x: f32,
        /// The magnification of the camera in the y-direction
        ///
        /// Basically the height of the viewing volume
        mag_y: f32,
        /// Coordinate of the near clipping plane on the camera's local z-axis
        near_z: f32,
        /// Coordinate of the far clipping plane on the camera's local z-axis
        far_z: f32,
    },
}

impl<'a> From<gltf::Camera<'a>> for CameraType {
    fn from(cam: gltf::Camera<'a>) -> Self {
        use gltf::camera::Projection::*;
        match cam.projection() {
            Perspective(persp) => CameraType::Perspective {
                aspect_ratio: persp.aspect_ratio().unwrap_or(0.0),
                field_of_view_y: Radians::from_radians(persp.yfov()),
                near_z: persp.znear(),
                far_z: persp.zfar(),
            },

            Orthographic(ortho) => CameraType::Orthographic {
                mag_x: ortho.xmag(),
                mag_y: ortho.ymag(),
                near_z: ortho.znear(),
                far_z: ortho.zfar(),
            },
        }
    }
}

impl CameraType {
    /// The perspective/orthographic projection matrix of the camera.
    ///
    /// Camera coordinates -> Homogenous coordinates
    pub fn to_projection(&self) -> Mat4 {
        // OpenGL clip planes are -1 to 1, thus we use the _no method
        use CameraType::*;
        match *self {
            Perspective {aspect_ratio, field_of_view_y, near_z, far_z} => match far_z {
                Some(far_z) => {
                    Mat4::perspective_rh_no(field_of_view_y.get_radians(), aspect_ratio, near_z, far_z)
                },

                None => {
                    // Infinite projection matrix
                    // Source: http://www.terathon.com/gdc07_lengyel.pdf
                    let focal_length = 1.0 / (field_of_view_y.get_radians() / 2.0).tan();
                    Mat4::new(
                        focal_length,   0.0,                            0.0,    0.0,
                        0.0,            focal_length / aspect_ratio,    0.0,    0.0,
                        0.0,            0.0,                           -1.0,   -2.0*near_z,
                        0.0,            0.0,                           -1.0,    0.0,
                    )
                },
            },

            Orthographic {mag_x, mag_y, near_z, far_z} => {
                Mat4::orthographic_rh_no(FrustumPlanes {
                    left: -mag_x/2.0,
                    right: mag_x/2.0,
                    bottom: -mag_y/2.0,
                    top: mag_y/2.0,
                    near: near_z,
                    far: far_z,
                })
            },
        }
    }
}
