use glium::uniforms::{Uniforms, UniformValue, Sampler};

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
            let &ShaderTexture {ref image, magnify_filter, minify_filter, wrap_function} = texture;

            let sampler = image.sampled();
            let sampler = magnify_filter.map(|filter| sampler.magnify_filter(filter))
                .unwrap_or(sampler);
            let sampler = minify_filter.map(|filter| sampler.minify_filter(filter))
                .unwrap_or(sampler);
            let sampler = sampler.wrap_function(wrap_function);

            let Sampler(_, behaviour) = sampler;
            UniformValue::Texture2d(image, Some(behaviour))
        });

        Self {
            diffuse_color: UniformValue::Vec4(diffuse_color.into_array()),
            sampler,
        }
    }
}
