use std::sync::Arc;

use glium::uniforms::{Uniforms, UniformValue};

use crate::math::{Mat4, Rgb};
use crate::renderer::{Light, JointMatrixTexture, ShaderMaterial};

use super::nested_uniforms::NestedUniforms;
use super::light_uniform::LightUniform;
use super::material_uniform::MaterialUniform;

/// The maximum supported number of lights
///
/// This value must match the corresponding value in the cel shaders
const MAX_LIGHTS: usize = 10;

pub struct CelUniforms<'a> {
    pub mvp: Mat4,
    pub model_transform: Mat4,
    pub model_inverse_transpose: Mat4,
    pub joint_matrices: &'a JointMatrixTexture,
    pub lights: &'a [Arc<Light>],
    pub ambient_light: Rgb,
    pub material: &'a ShaderMaterial,
}

/// This struct must match the uniforms in the cel shaders
pub struct Cel<'a> {
    mvp: UniformValue<'static>,
    model_transform: UniformValue<'static>,
    model_inverse_transpose: UniformValue<'static>,
    joint_matrices: UniformValue<'a>,
    num_lights: UniformValue<'static>,
    lights: Vec<LightUniform>,
    ambient_light: UniformValue<'static>,
    material: MaterialUniform<'a>,
}

impl<'b> Uniforms for Cel<'b> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let Self {
            mvp,
            model_transform,
            model_inverse_transpose,
            joint_matrices,
            num_lights,
            lights,
            ambient_light,
            material,
        } = self;

        visit("mvp", *mvp);
        visit("model_transform", *model_transform);
        visit("model_inverse_transpose", *model_inverse_transpose);
        visit("joint_matrices", *joint_matrices);
        visit("num_lights", *num_lights);
        for (i, light) in lights.iter().enumerate() {
            light.visit_nested_index("lights", i, &mut visit);
        }
        visit("ambient_light", *ambient_light);
        material.visit_nested("material", &mut visit);
    }
}

impl<'a> From<CelUniforms<'a>> for Cel<'a> {
    fn from(cel_uniforms: CelUniforms<'a>) -> Self {
        let CelUniforms {
            mvp,
            model_transform,
            model_inverse_transpose,
            joint_matrices,
            lights,
            ambient_light,
            material,
        } = cel_uniforms;

        assert!(lights.len() <= MAX_LIGHTS, "Only up to {} lights can be rendered at any given time", MAX_LIGHTS);

        Self {
            mvp: UniformValue::Mat4(mvp.into_col_arrays()),
            model_transform: UniformValue::Mat4(model_transform.into_col_arrays()),
            model_inverse_transpose: UniformValue::Mat4(model_inverse_transpose.into_col_arrays()),
            joint_matrices: UniformValue::Texture2d(joint_matrices.as_texture(), None),
            num_lights: UniformValue::SignedInt(lights.len() as i32),
            lights: lights.iter().map(|light| {
                let Light {data, world_transform} = &**light;
                LightUniform::new(data, *world_transform)
            }).collect(),
            ambient_light: UniformValue::Vec3(ambient_light.into_array()),
            material: MaterialUniform::new(material),
        }
    }
}
