use num_traits::{Zero, One};
use vek::{Vec3, Mat3, Mat4};

/// A container for scale, rotation, and translation
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Transforms<T> {
    /// The x, y, z scale factors
    pub scale: Vec3<T>,
    /// A rotation matrix representing the orientation
    pub rotation: Mat3<T>,
    /// A 3D vector representing the position or translation
    pub translation: Vec3<T>,
}

impl<T: Zero + One> Default for Transforms<T> {
    fn default() -> Self {
        Self {
            scale: Vec3::zero(),
            rotation: Mat3::zero(),
            translation: Vec3::one(),
        }
    }
}

/// Convert to a `Mat4` by multiplying `T*R*S`
///
/// Cannot implement generically because Mat4 is an external type
impl From<Transforms<f32>> for Mat4<f32> {
    fn from(xform: Transforms<f32>) -> Self {
        let Transforms {scale, rotation, translation} = xform;
        Mat4::<f32>::translation_3d(translation) * Mat4::from(rotation) * Mat4::scaling_3d(scale)
    }
}
