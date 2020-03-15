use glium::uniforms::{Uniforms, UniformValue, SamplerBehavior};

use crate::renderer::{ShaderMaterial, ShaderTexture};

/// This struct must match the `Material` struct in our shaders
pub struct MaterialUniform<'a> {
    diffuse_color: UniformValue<'static>,
    sampler: Option<UniformValue<'a>>,
}

impl<'b> Uniforms for MaterialUniform<'b> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let &Self {diffuse_color, sampler} = self;
        visit("diffuse_color", diffuse_color);
        if let Some(sampler) = sampler {
            visit("sampler", sampler);
        }
    }
}

impl<'a> MaterialUniform<'a> {
    pub fn new(material: &'a ShaderMaterial) -> Self {
        let &ShaderMaterial {diffuse_color, ref texture} = material;

        let sampler = texture.as_ref().map(|texture| {
            let &ShaderTexture {ref image, magnify_filter, minify_filter, wrap_s, wrap_t} = texture;

            let mut behavior = SamplerBehavior::default();
            if let Some(magnify_filter) = magnify_filter {
                behavior.magnify_filter = magnify_filter;
            }
            if let Some(minify_filter) = minify_filter {
                behavior.minify_filter = minify_filter;
            }
            behavior.wrap_function.0 = wrap_s;
            behavior.wrap_function.1 = wrap_t;

            UniformValue::Texture2d(image, Some(behavior))
        });

        Self {
            diffuse_color: UniformValue::Vec4(diffuse_color.into_array()),
            sampler,
        }
    }
}
