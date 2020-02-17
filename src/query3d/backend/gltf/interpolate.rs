use crate::math::{Vec3, Quaternion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interpolation {
    Linear,
    Step,
}

impl From<gltf::animation::Interpolation> for Interpolation {
    fn from(interp: gltf::animation::Interpolation) -> Self {
        use gltf::animation::Interpolation::*;
        match interp {
            Linear => Interpolation::Linear,
            Step => Interpolation::Step,
            //TODO - In order to support cubicspline interpolation, we need to change how we're
            // storing the data
            // https://github.com/KhronosGroup/glTF/tree/master/specification/2.0#animation-samplerinterpolation
            CubicSpline => unimplemented!("Cubicspline interpolation is not supported!"),
        }
    }
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
