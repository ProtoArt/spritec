use glium::uniforms::{Uniforms, UniformValue};

use crate::model::Material;

/// This struct must match the `Material` struct in our shaders
pub struct MaterialUniform {
    diffuse_color: UniformValue<'static>,
}

impl Uniforms for MaterialUniform {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let &Self {diffuse_color} = self;
        visit("diffuse_color", diffuse_color);
    }
}

impl MaterialUniform {
    pub fn new(material: &Material) -> Self {
        let &Material {diffuse_color} = material;

        Self {
            diffuse_color: UniformValue::Vec4(diffuse_color.into_array()),
        }
    }
}
