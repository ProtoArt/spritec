use crate::math::Mat4;
use glium::uniforms::{Uniforms, UniformValue};

use crate::scene::Material;
use crate::renderer::Light;

use super::nested_uniforms::NestedUniforms;
pub use super::light_uniform::LightUniform;
pub use super::material_uniform::MaterialUniform;

pub struct CelUniforms<'a> {
    pub mvp: Mat4,
    pub model_transform: Mat4,
    pub model_inverse_transpose: Mat4,
    pub light: &'a Light,
    pub light_world_transform: Mat4,
    pub ambient_intensity: f32,
    pub material: &'a Material,
}

/// This struct must match the uniforms in the cel shaders
pub struct Cel {
    mvp: UniformValue<'static>,
    model_transform: UniformValue<'static>,
    model_inverse_transpose: UniformValue<'static>,
    light: LightUniform,
    ambient_intensity: UniformValue<'static>,
    material: MaterialUniform,
}

impl Uniforms for Cel {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let Self {
            mvp,
            model_transform,
            model_inverse_transpose,
            light,
            ambient_intensity,
            material,
        } = self;

        visit("mvp", *mvp);
        visit("model_transform", *model_transform);
        visit("model_inverse_transpose", *model_inverse_transpose);
        light.visit_nested("light", &mut visit);
        visit("ambient_intensity", *ambient_intensity);
        material.visit_nested("material", &mut visit);
    }
}

impl<'a> From<CelUniforms<'a>> for Cel {
    fn from(cel_uniforms: CelUniforms<'a>) -> Self {
        let CelUniforms {
            mvp,
            model_transform,
            model_inverse_transpose,
            light,
            light_world_transform,
            ambient_intensity,
            material,
        } = cel_uniforms;

        Self {
            mvp: UniformValue::Mat4(mvp.into_col_arrays()),
            model_transform: UniformValue::Mat4(model_transform.into_col_arrays()),
            model_inverse_transpose: UniformValue::Mat4(model_inverse_transpose.into_col_arrays()),
            light: LightUniform::new(light, light_world_transform),
            ambient_intensity: UniformValue::Float(ambient_intensity),
            material: MaterialUniform::new(material),
        }
    }
}
