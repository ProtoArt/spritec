use std::sync::Arc;

use glium::{
    Texture2d,
    texture::TextureCreationError,
    uniforms::{SamplerWrapFunction, MinifySamplerFilter, MagnifySamplerFilter},
};

use crate::math::Rgba;
use crate::scene::{Material, Texture, TexImage};

/// A material that can be used on the GPU
#[derive(Debug)]
pub struct ShaderMaterial {
    pub diffuse_color: Rgba,
    pub texture: Option<ShaderTexture>,
}

impl ShaderMaterial {
    pub fn new(
        material: &Material,
        image_lookup: impl FnMut(&TexImage) -> Result<&Arc<Texture2d>, TextureCreationError>,
    ) -> Result<Self, TextureCreationError> {
        let &Material {diffuse_color, ref texture} = material;
        let texture = texture.as_ref()
            .map(|tex| ShaderTexture::new(tex, image_lookup))
            .transpose()?;

        Ok(Self {diffuse_color, texture})
    }
}

/// A texture stored on the GPU
#[derive(Debug)]
pub struct ShaderTexture {
    pub image: Arc<Texture2d>,
    pub magnify_filter: Option<MagnifySamplerFilter>,
    pub minify_filter: Option<MinifySamplerFilter>,
    pub wrap_function: SamplerWrapFunction,
}

impl ShaderTexture {
    pub fn new(
        texture: &Texture,
        mut image_lookup: impl FnMut(&TexImage) -> Result<&Arc<Texture2d>, TextureCreationError>,
    ) -> Result<Self, TextureCreationError> {
        let &Texture {ref image, magnify_filter, minify_filter, wrap_function} = texture;

        Ok(Self {
            image: image_lookup(image)?.clone(),
            magnify_filter: magnify_filter.map(|filter| filter.into()),
            minify_filter: minify_filter.map(|filter| filter.into()),
            wrap_function: wrap_function.into(),
        })
    }
}
