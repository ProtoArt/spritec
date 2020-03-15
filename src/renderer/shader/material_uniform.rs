use glium::uniforms::{Uniforms, UniformValue};

use crate::renderer::{ShaderMaterial, ShaderTexture};

/// This struct must match the `Material` struct in our shaders
pub struct MaterialUniform<'a> {
    diffuse_color: UniformValue<'a>,
}

impl<'b> Uniforms for MaterialUniform<'b> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let &Self {diffuse_color} = self;
        visit("diffuse_color", diffuse_color);
    }
}

impl<'a> MaterialUniform<'a> {
    pub fn new(material: &'a ShaderMaterial) -> Self {
        let &ShaderMaterial {diffuse_color, ref texture} = material;

        let texture = texture.as_ref().map(|texture| {
            let &ShaderTexture {ref image, magnify_filter, minify_filter, wrap_function} = texture;

            todo!()
        });

        Self {
            diffuse_color: UniformValue::Vec4(diffuse_color.into_array()),
        }
    }
}
