use crate::math::{Vec3, Quaternion};

#[derive(Debug)]
pub enum Interpolation {
    Linear,
    Step,
}

pub trait Interpolate {
    fn interpolate(interp: &Interpolation, t: f32, prev_keyframe: Self, next_keyframe: Self) -> Self;
}

impl Interpolate for Vec3 {
    fn interpolate(interp: &Interpolation, t: f32, prev_keyframe: Vec3, next_keyframe: Vec3) -> Vec3 {
        use Interpolation::*;
        match interp {
            Linear => {
                let [x, y, z] = interpolation::lerp(&prev_keyframe.into_array(), &next_keyframe.into_array(), &t);
                Vec3 {x, y, z}
            },
            Step => prev_keyframe,
        }
    }
}

impl Interpolate for Quaternion {
    fn interpolate(interp: &Interpolation, t: f32, prev_keyframe: Quaternion, next_keyframe: Quaternion) -> Quaternion {
        use Interpolation::*;
        match interp {
            Linear => Quaternion::slerp(prev_keyframe, next_keyframe, t),
            Step => prev_keyframe,
        }
    }
}
