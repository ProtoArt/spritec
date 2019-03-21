use vek::{Mat4, Vec3, Vec4, Rgba};
use euc::{Pipeline, DepthStrategy};

use crate::color::rgba_to_bgra_u32;
use crate::geometry::Mesh;

/// An outline shader
/// Initial version based on this article: http://rbwhitaker.wikidot.com/toon-shader
///
/// Global assumptions:
/// * Color values (red, green, blue, alpha) are all between 0.0 and 1.0
/// * Direction vectors are normalized
#[derive(Debug)]
pub struct OutlineShader<'a> {
    /// The model-view-projection matrix
    pub mvp: Mat4<f32>,

    /// The input to the shader
    pub mesh: &'a Mesh,

    /// The color for drawing the outline
    pub outline_color: Rgba<f32>,
    /// The thickness of the outlines. This may need to change, depending on the scale of the
    /// objects you are drawing.
    pub outline_thickness: f32,
}

impl<'a> Pipeline for OutlineShader<'a> {
    type Vertex = u32; // Vertex index
    type VsOut = Vec3<f32>; // Normal
    type Pixel = u32; // BGRA

    /// The vertex shader that does the outlines.
    #[inline(always)]
    fn vert(&self, v_index: &Self::Vertex) -> ([f32; 3], Self::VsOut) {
        let v_index = *v_index as usize;
        // Find vertex position
        let v_pos = Vec4::from_point(self.mesh.position(v_index));
        // Calculate vertex position in camera space
        let v_pos_cam = Vec3::from(self.mvp * v_pos);
        // Find vertex normal
        let v_norm = Vec4::from_point(self.mesh.normal(v_index));
        // Transform normals to preserve orthogonality after non-uniform transformation
        let v_norm_cam = Vec3::from(self.mvp * v_norm);

        // Take the correct "original" location and translate the vertex a little bit in the
        // direction of the normal to draw a slightly expanded object. Later, we will draw over
        // most of this with the right color, except the expanded part, which will leave the
        // outline that we want.
        let v_pos_outline = v_pos_cam + v_norm_cam * self.outline_thickness;

        (v_pos_outline.into_array(), v_norm_cam)
    }

    /// The fragment/pixel shader for the outline.
    #[inline(always)]
    fn frag(&self, _: &Self::VsOut) -> Self::Pixel {
        // Draw everything with the correct line color
        let color = self.outline_color;

        rgba_to_bgra_u32(color)
    }

    fn get_depth_strategy(&self) -> DepthStrategy {
        DepthStrategy::IfLessNoWrite
    }
}
