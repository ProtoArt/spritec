use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use vek::Rgba;

use crate::config;
use crate::camera::Camera;
use crate::light::DirectionalLight;
use crate::query3d::{GeometryQuery, LightQuery, CameraQuery, File};

#[derive(Debug)]
pub struct Render {
    /// The size at which to render the generated image
    pub size: Size,
    /// A scale factor to apply to the generated image. The image is scaled without interpolation.
    /// The value must be greater than zero.
    pub scale: NonZeroU32,
    /// The background color of the generated image
    pub background: Rgba<f32>,
    /// The camera perspective from which to render each frame
    pub camera: RenderCamera,
    /// The lights to use to light the rendered scene
    pub lights: Vec<RenderLight>,
    /// The models to draw in the rendered image
    pub models: Vec<RenderModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

#[derive(Debug)]
pub struct RenderModel {
    pub model: FileQuery<GeometryQuery>,
    pub outline: Outline,
}

#[derive(Debug, Clone)]
pub struct Outline {
    /// The outline thickness to use when drawing the generated image
    ///
    /// The value must not be negative.
    pub thickness: f32,
    /// The color of the outline to draw
    pub color: Rgba<f32>,
}

impl From<config::Outline> for Outline {
    fn from(outline: config::Outline) -> Self {
        let config::Outline {thickness, color} = outline;

        Self {thickness, color}
    }
}

#[derive(Debug)]
pub enum RenderCamera {
    Camera(Camera),
    Query(FileQuery<CameraQuery>),
}

#[derive(Debug)]
pub enum RenderLight {
    Light(DirectionalLight),
    Query(FileQuery<LightQuery>),
}

#[derive(Debug)]
pub struct FileQuery<Q> {
    pub query: Q,
    pub file: Arc<Mutex<File>>,
}
