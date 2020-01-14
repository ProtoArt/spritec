use std::iter::Sum;

use num_traits::real::Real;
use vek::{Vec3, Mat3, Mat4, ops::MulAdd};

use super::transforms::Transforms;

/// Extension trait that allows you to decompose a transformation matrix into its scale, rotation,
/// and translation components.
pub trait Decompose<T> {
    /// Decomposes this matrix into its translation, rotation, and scale components.
    fn decompose(self) -> Transforms<T>;
}

impl<T: Real + MulAdd<T, T, Output=T> + Sum> Decompose<T> for Mat4<T> {
    /// Decomposes this matrix into its translation, rotation, and scale components.
    ///
    /// The implementation is very basic and does little error detection. Since it
    /// returns `Transforms`, we do not produce any skew or perspective information (common
    /// in libraries like glm). That means that you should only expect good results for
    /// matrices you know are composed of only valid scales, rotations, and translations.
    ///
    /// # Example
    ///
    /// ```
    /// # use approx::{assert_relative_eq, relative_eq};
    /// # use vek::{Transform, Mat3, Mat4, Quaternion, Vec3};
    /// use spritec::math::{Decompose, Transforms};
    ///
    /// let (p, rz, s) = (Vec3::unit_x(), 3.0_f32, 5.0_f32);
    /// let a = Mat4::scaling_3d(s).rotated_z(rz).translated_3d(p);
    /// let b = Transforms {
    ///     scale: Vec3::broadcast(s),
    ///     rotation: Mat3::rotation_z(rz),
    ///     translation: p,
    /// };
    ///
    /// let a_decomp = a.decompose();
    /// assert_relative_eq!(a_decomp.scale, b.scale);
    /// assert_relative_eq!(a_decomp.rotation, b.rotation);
    /// assert_relative_eq!(a_decomp.translation, b.translation);
    /// ```
    fn decompose(self) -> Transforms<T> {
        // See: https://math.stackexchange.com/a/1463487/161715

        let translation = Vec3 {
            x: self[(0, 3)],
            y: self[(1, 3)],
            z: self[(2, 3)],
        };

        let sx = Vec3 {x: self[(0, 0)], y: self[(1, 0)], z: self[(2, 0)]}.magnitude();
        let sy = Vec3 {x: self[(0, 1)], y: self[(1, 1)], z: self[(2, 1)]}.magnitude();
        let sz = Vec3 {x: self[(0, 2)], y: self[(1, 2)], z: self[(2, 2)]}.magnitude();

        let scale = Vec3 {x: sx, y: sy, z: sz};

        let rotation = Mat3::new(
            self[(0, 0)] / sx, self[(0, 1)] / sy, self[(0, 2)] / sz,
            self[(1, 0)] / sx, self[(1, 1)] / sy, self[(1, 2)] / sz,
            self[(2, 0)] / sx, self[(2, 1)] / sy, self[(2, 2)] / sz,
        );

        Transforms {scale, rotation, translation}
    }
}
