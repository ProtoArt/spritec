use std::sync::Arc;

use gltf::image::Data as ImageData;

// NOTE: Normally, we would want to use encapsulation to protect this ID and make sure that
// it is valid. In this case though, it's tough to do that in a meaningful way because we
// go through a Vec<gltf::image::Data> and the `Data` struct has no `index` method. The result
// of this is that we're basically just relying on users of this ID to maintain that it's valid.
// That's not super great, but there doesn't seem to be a better option here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageId(pub usize);

#[derive(Debug)]
pub struct TexImage {
    pub id: ImageId,
    pub data: ImageData,
}

/// Magnification filter
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MagFilter {
    Nearest,
    Linear,
}

impl From<gltf::texture::MagFilter> for MagFilter {
    fn from(mode: gltf::texture::MagFilter) -> Self {
        match mode {
            gltf::texture::MagFilter::Nearest => MagFilter::Nearest,
            gltf::texture::MagFilter::Linear => MagFilter::Linear,
        }
    }
}

/// Minification filter
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

impl From<gltf::texture::MinFilter> for MinFilter {
    fn from(mode: gltf::texture::MinFilter) -> Self {
        match mode {
            gltf::texture::MinFilter::Nearest => MinFilter::Nearest,
            gltf::texture::MinFilter::Linear => MinFilter::Linear,
            gltf::texture::MinFilter::NearestMipmapNearest => MinFilter::NearestMipmapNearest,
            gltf::texture::MinFilter::LinearMipmapNearest => MinFilter::LinearMipmapNearest,
            gltf::texture::MinFilter::NearestMipmapLinear => MinFilter::NearestMipmapLinear,
            gltf::texture::MinFilter::LinearMipmapLinear => MinFilter::LinearMipmapLinear,
        }
    }
}

/// Texture coordinate wrapping mode
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WrappingMode {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}

impl From<gltf::texture::WrappingMode> for WrappingMode {
    fn from(mode: gltf::texture::WrappingMode) -> Self {
        match mode {
            gltf::texture::WrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
            gltf::texture::WrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
            gltf::texture::WrappingMode::Repeat => WrappingMode::Repeat,
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    pub image: Arc<TexImage>,
    pub magnify_filter: Option<MagFilter>,
    pub minify_filter: Option<MinFilter>,
    pub wrap_function: WrappingMode,
}

impl Texture {
    pub fn from_gltf(tex: gltf::Texture, images: &[Arc<TexImage>]) -> Self {
        let sampler = tex.sampler();
        assert_eq!(sampler.wrap_s(), sampler.wrap_t(),
            "bug: separate s and t wrap functions are not supported in glTF models");

        Self {
            image: images[tex.source().index()].clone(),
            magnify_filter: sampler.mag_filter().map(|f| f.into()),
            minify_filter: sampler.min_filter().map(|f| f.into()),
            wrap_function: sampler.wrap_s().into(),
        }
    }
}
