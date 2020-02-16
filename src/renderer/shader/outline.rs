use glium::uniforms::{Uniforms, UniformValue};

use crate::renderer::JointMatrixTexture;
use crate::math::{Mat4, Rgba};

pub struct OutlineUniforms<'a> {
    pub mvp: Mat4,
    pub joint_matrices: &'a JointMatrixTexture,
    pub outline_thickness: f32,
    pub outline_color: Rgba,
}

/// This struct must match the uniforms in the outline shaders
pub struct Outline<'a> {
    mvp: UniformValue<'static>,
    joint_matrices: UniformValue<'a>,
    outline_thickness: UniformValue<'static>,
    outline_color: UniformValue<'static>,
}

impl<'b> Uniforms for Outline<'b> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let &Self {mvp, joint_matrices, outline_thickness, outline_color} = self;

        visit("mvp", mvp);
        visit("joint_matrices", joint_matrices);
        visit("outline_thickness", outline_thickness);
        visit("outline_color", outline_color);
    }
}

impl<'a> From<OutlineUniforms<'a>> for Outline<'a> {
    fn from(outline_uniforms: OutlineUniforms<'a>) -> Self {
        let OutlineUniforms {mvp, joint_matrices, outline_thickness, outline_color} = outline_uniforms;

        Self {
            mvp: UniformValue::Mat4(mvp.into_col_arrays()),
            joint_matrices: UniformValue::Texture2d(joint_matrices.as_texture(), None),
            outline_thickness: UniformValue::Float(outline_thickness),
            outline_color: UniformValue::Vec4(outline_color.into_array()),
        }
    }
}
