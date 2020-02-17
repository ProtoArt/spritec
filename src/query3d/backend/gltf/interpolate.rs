use crate::math::{Vec3, Quaternion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interpolation {
    Linear,
    Step,
}

pub trait Interpolate {
    /// Interpolate between two values using the given method.
    ///
    /// `weight` is always between 0.0 and 1.0
    fn interpolate(method: Interpolation, weight: f32, start: &Self, end: &Self) -> Self;
}

impl Interpolate for Vec3 {
    fn interpolate(method: Interpolation, weight: f32, start: &Vec3, end: &Vec3) -> Vec3 {
        use Interpolation::*;
        match method {
            Linear => {
                let start = start.into_array();
                let end = end.into_array();
                let [x, y, z] = interpolation::lerp(&start, &end, &weight);

                Vec3 {x, y, z}
            },
            Step => *start,
        }
    }
}

impl Interpolate for Quaternion {
    fn interpolate(method: Interpolation, weight: f32, start: &Quaternion, end: &Quaternion) -> Quaternion {
        use Interpolation::*;
        match method {
            Linear => Quaternion::slerp(*start, *end, weight),
            Step => *start,
        }
    }
}
