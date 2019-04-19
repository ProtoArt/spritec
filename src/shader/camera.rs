use std::f32::consts::PI;

use vek::Mat4;

use crate::config;

#[derive(Debug, Clone)]
pub struct Camera {
    //TODO(#4): These fields should be replaced with: position, target, fov, aspect_ratio_x, etc.
    pub(crate) view: Mat4<f32>,
    pub(crate) projection: Mat4<f32>,
}

impl From<config::PresetCamera> for Camera {
    fn from(cam: config::PresetCamera) -> Self {
        use config::PresetCamera::*;
        match cam {
            Perspective(persp) => persp.into(),
            //TODO(#4): This should be implemented as part of #4.
            Custom {position, target} => unimplemented!(),
        }
    }
}

impl From<config::Perspective> for Camera {
    fn from(persp: config::Perspective) -> Self {
        //TODO(#4): This should be reimplemented properly as part of #4. These placeholder values
        // are only meant to work for the desired angles of bigboi. The angles are slightly tilted.
        // In the actual implementation they should be straight on.
        use config::Perspective::*;
        let view = match persp {
            PerspectiveFront => Mat4::rotation_x(PI/8.0) * Mat4::rotation_y(0.0*PI/2.0),
            PerspectiveBack => unimplemented!("TODO"),
            PerspectiveLeft => unimplemented!("TODO"),
            PerspectiveRight => Mat4::rotation_x(PI/8.0) * Mat4::rotation_y(-1.0*PI/2.0),
            PerspectiveTop => unimplemented!("TODO"),
            PerspectiveBottom => unimplemented!("TODO"),
        };

        //TODO(#4): This should be implemented as part of #4. We may want to add some additional
        // settings to the Custom variant of PresetCamera as well as additional fields to Camera.
        // The variables below are good examples of what these additional fields could be called:
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
