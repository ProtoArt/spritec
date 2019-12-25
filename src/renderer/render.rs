use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use vek::Rgba;

use crate::config;
use crate::camera::Camera;
use crate::light::DirectionalLight;
use crate::query3d::{GeometryQuery, LightQuery, CameraQuery, File, QueryError};

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
    pub models: Vec<RenderGeometry>,
}

impl Render {
    pub fn total_size(&self) -> Size {
        let scale = self.scale.get();
        // This code is safe because two non-zero values multiplied by each other is still non-zero
        Size {
            width: unsafe { NonZeroU32::new_unchecked(self.size.width.get() * scale) },
            height: unsafe { NonZeroU32::new_unchecked(self.size.height.get() * scale) },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

impl Size {
    pub fn min_value() -> Self {
        // This code is safe because both values are non-zero
        Self {
            width: unsafe { NonZeroU32::new_unchecked(1) },
            height: unsafe { NonZeroU32::new_unchecked(1) },
        }
    }
}

impl Size {
    pub fn max(self, other: Self) -> Self {
        let Self {width, height} = self;

        // This code is safe because all the values involved are non-zero
        Self {
            width: unsafe { NonZeroU32::new_unchecked(width.get().max(other.width.get())) },
            height: unsafe { NonZeroU32::new_unchecked(height.get().max(other.height.get())) },
        }
    }
}

#[derive(Debug)]
pub struct RenderGeometry {
    pub geometry: FileQuery<GeometryQuery>,
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

impl RenderCamera {
    pub fn to_camera(self) -> Result<Camera, QueryError> {
        use RenderCamera::*;
        match self {
            Camera(cam) => Ok(cam),
            Query(_) => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub enum RenderLight {
    Light(DirectionalLight),
    Query(FileQuery<LightQuery>),
}

impl RenderLight {
    pub fn to_light(self) -> Result<DirectionalLight, QueryError> {
        use RenderLight::*;
        match self {
            Light(light) => Ok(light),
            Query(_) => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct FileQuery<Q> {
    pub query: Q,
    pub file: Arc<Mutex<File>>,
}
