use glium::uniforms::{Uniforms, UniformValue};

use crate::renderer::DirectionalLight;

/// This struct must match the `DirectionalLight` struct in our shaders
pub struct DirectionalLightUniform {
    direction: UniformValue<'static>,
    color: UniformValue<'static>,
    intensity: UniformValue<'static>,
}

impl Uniforms for DirectionalLightUniform {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let &Self {direction, color, intensity} = self;
        visit("direction", direction);
        visit("color", color);
        visit("intensity", intensity);
    }
}

impl DirectionalLightUniform {
    pub fn new(light: &DirectionalLight) -> Self {
        let DirectionalLight {direction, color, intensity} = light;
        Self {
            direction: UniformValue::Vec3(direction.into_array()),
            color: UniformValue::Vec4(color.into_array()),
            intensity: UniformValue::Float(*intensity),
        }
    }
}
