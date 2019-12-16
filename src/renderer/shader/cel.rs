use vek::Mat4;
use glium::uniforms::{Uniforms, UniformValue};

use crate::light::DirectionalLight;
use crate::model::Material;

use super::nested_uniforms::NestedUniforms;
pub use super::light_uniform::DirectionalLightUniform;
pub use super::material_uniform::MaterialUniform;

pub struct CelUniforms<'a> {
    pub mvp: Mat4<f32>,
    pub model_view_inverse_transpose: Mat4<f32>,
    pub light: &'a DirectionalLight,
    pub ambient_intensity: f32,
    pub material: &'a Material,
}

/// This struct must match the uniforms in the cel shaders
pub struct Cel {
    mvp: UniformValue<'static>,
    model_view_inverse_transpose: UniformValue<'static>,
    light: NestedUniforms<DirectionalLightUniform>,
    ambient_intensity: UniformValue<'static>,
    material: NestedUniforms<MaterialUniform>,
}

impl Uniforms for Cel {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let Self {mvp, model_view_inverse_transpose, light, ambient_intensity, material} = self;

        visit("mvp", *mvp);
        visit("model_view_inverse_transpose", *model_view_inverse_transpose);
        light.visit_nested("light", &mut visit);
        visit("ambient_intensity", *ambient_intensity);
        material.visit_nested("material", &mut visit);
    }
}

impl<'a> From<CelUniforms<'a>> for Cel {
    fn from(cel_uniforms: CelUniforms<'a>) -> Self {
        let CelUniforms {mvp, model_view_inverse_transpose, light, ambient_intensity, material} = cel_uniforms;

        Self {
            mvp: UniformValue::Mat4(mvp.into_col_arrays()),
            model_view_inverse_transpose: UniformValue::Mat4(model_view_inverse_transpose.into_col_arrays()),
            light: NestedUniforms::new(DirectionalLightUniform::new(light)),
            ambient_intensity: UniformValue::Float(ambient_intensity),
            material: NestedUniforms::new(MaterialUniform::new(material)),
        }
    }
}
