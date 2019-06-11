use vek::{Mat4, Vec3, Vec4, Rgba, Clamp};
use euc::Pipeline;

use crate::shader::DiffuseLight;
use crate::model::Mesh;

/// A Cel/Toon shader implementation
/// Initial version based on this article: http://rbwhitaker.wikidot.com/toon-shader
///
/// Global assumptions:
/// * Color values (red, green, blue, alpha) are all between 0.0 and 1.0
/// * Direction vectors are normalized
/// * Y-up, right-handed coordinate system
#[derive(Debug)]
pub struct CelShader<'a> {
    /// The model-view-projection matrix
    pub mvp: Mat4<f32>,
    /// The transpose of the inverse of the world transformation, used for transforming the
    /// vertex's normal
    pub model_inverse_transpose: Mat4<f32>,

    /// The input to the shader
    pub mesh: &'a Mesh,

    // Diffuse light properties
    pub light: DiffuseLight,
    // Ambient light properties
    pub ambient_intensity: f32,
}

impl<'a> Pipeline for CelShader<'a> {
    // Fragment shader output
    type Pixel = Rgba<f32>;
    // Vertex index
    type Vertex = u32;
    // Normal
    type VsOut = Vec3<f32>;

    /// The vertex shader that does cel shading.
    ///
    /// It really only does the basic transformation of the vertex location, and normal, and copies
    /// the texture coordinate over.
    #[inline(always)]
    fn vert(&self, v_index: &Self::Vertex) -> ([f32; 3], Self::VsOut) {
        let v_index = *v_index as usize;
        // Find vertex position
        let v_pos = Vec4::from_point(self.mesh.position(v_index));
        // Calculate vertex position in camera space
        let v_pos_cam = Vec3::from(self.mvp * v_pos).into_array();
        // Find vertex normal
        let v_norm = Vec4::from_point(self.mesh.normal(v_index));
        // Transform normals to preserve orthogonality after non-uniform transformations.
        let v_norm = Vec3::from((self.model_inverse_transpose * v_norm).normalized());

        (v_pos_cam, v_norm)
    }

    /// The fragment/pixel shader that does cel shading. Basically, it calculates the color like it
    /// should, and then it discretizes the color into one of four colors.
    #[inline(always)]
    fn frag(&self, norm: &Self::VsOut) -> Self::Pixel {
        // Calculate diffuse light amount
        // max() is used to bottom out at zero if the dot product is negative
        let diffuse_intensity = norm.dot(self.light.direction).max(0.0);

        // The color of the material for this mesh
        let mat_color = self.mesh.material().diffuse_color;

        // Calculate what would normally be the final color, including texturing and diffuse lighting
        let light_intensity = self.ambient_intensity + diffuse_intensity;
        let color = mat_color * self.light.intensity;

        // Discretize the intensity, based on a few cutoff points
        let alpha = color.a;
        let mut final_color = match light_intensity {
            intensity if intensity > 0.95 => color,
            intensity if intensity > 0.5 => color * 0.7,
            intensity if intensity > 0.05 => color * 0.35,
            _ => color * 0.1,
        };

        // Gamma correction
        // Technique from: https://learnopengl.com/Advanced-Lighting/Gamma-Correction
        let gamma = 2.2;
        final_color = final_color.map(|c| c.powf(1.0/gamma));

        // Reassign the final alpha because we don't actually want to calculations above to
        // influence this value
        final_color.a = alpha;

        // Clamp the color values between 0.0 and 1.0
        let final_color = final_color.clamped(Rgba::zero(), Rgba::one());

        final_color
    }
}
