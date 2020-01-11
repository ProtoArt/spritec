use crate::math::{Mat4, Rgba};
use glium::uniforms::{Uniforms, UniformValue};

pub struct OutlineUniforms {
    pub mvp: Mat4,
    pub outline_thickness: f32,
    pub outline_color: Rgba,
}

/// This struct must match the uniforms in the outline shaders
pub struct Outline {
    mvp: UniformValue<'static>,
    outline_thickness: UniformValue<'static>,
    outline_color: UniformValue<'static>,
}

impl Uniforms for Outline {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let &Self {mvp, outline_thickness, outline_color} = self;

        visit("mvp", mvp);
        visit("outline_thickness", outline_thickness);
        visit("outline_color", outline_color);
    }
}

impl From<OutlineUniforms> for Outline {
    fn from(outline_uniforms: OutlineUniforms) -> Self {
        let OutlineUniforms {mvp, outline_thickness, outline_color} = outline_uniforms;

        Self {
            mvp: UniformValue::Mat4(mvp.into_col_arrays()),
            outline_thickness: UniformValue::Float(outline_thickness),
            outline_color: UniformValue::Vec4(outline_color.into_array()),
        }
    }
}
