//! Math-related utilities/constants/type definitions.
//!
//! The vek crate that we use to provide math primitives is very generic, but we will always want
//! to use it with floats. This module exports type aliases that allow us to not have to specify
//! that we are using "f32" all the time.

mod decompose;

pub mod transforms;

pub use decompose::Decompose;

use serde::{Serialize, Deserialize};

pub type Vec2 = vek::Vec2<f32>;
pub type Vec3 = vek::Vec3<f32>;
pub type Vec4 = vek::Vec4<f32>;

pub type Mat2 = vek::Mat2<f32>;
pub type Mat3 = vek::Mat3<f32>;
pub type Mat4 = vek::Mat4<f32>;

pub type Quaternion = vek::Quaternion<f32>;

pub type Rgb = vek::Rgb<f32>;
pub type Rgba = vek::Rgba<f32>;

pub type FrustumPlanes = vek::FrustumPlanes<f32>;

pub type Transforms = transforms::Transforms<f32>;

/// A "newtype" to represent a value with the unit "radians"
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Radians(f32);

impl From<Degrees> for Radians {
    fn from(value: Degrees) -> Self {
        Radians::from_radians(value.get_radians())
    }
}

impl Radians {
    pub fn from_degrees(value: f32) -> Self {
        Radians(value.to_radians())
    }

    pub fn from_radians(value: f32) -> Self {
        Radians(value)
    }

    pub fn get_radians(self) -> f32 {
        self.0
    }

    pub fn get_degrees(self) -> f32 {
        self.0.to_degrees()
    }
}

/// A "newtype" to represent a value with the unit "degrees"
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Degrees(f32);

impl From<Radians> for Degrees {
    fn from(value: Radians) -> Self {
        Degrees::from_degrees(value.get_degrees())
    }
}

impl Degrees {
    pub fn from_degrees(value: f32) -> Self {
        Degrees(value)
    }

    pub fn from_radians(value: f32) -> Self {
        Degrees(value.to_degrees())
    }

    pub fn get_radians(self) -> f32 {
        self.0.to_radians()
    }

    pub fn get_degrees(self) -> f32 {
        self.0
    }
}
