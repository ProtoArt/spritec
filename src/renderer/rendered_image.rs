use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use crate::math::{Rgb, Rgba};

use crate::query3d::{GeometryQuery, LightQuery, CameraQuery, File, QueryError, QueryBackend};

use super::{Camera, Light};

/// An image that will be rendered using the given information
#[derive(Debug, Clone)]
pub struct RenderedImage {
    /// The size at which to render the generated image
    pub size: Size,
    /// The background color of the generated image
    pub background: Rgba,
    /// The camera perspective from which to render each frame
    pub camera: RenderCamera,
    /// The lights to use to light the rendered scene
    pub lights: RenderLights,
    /// The ambient light in the scene
    pub ambient_light: Rgb,
    /// The geometry to draw in the rendered image
    pub geometry: FileQuery<GeometryQuery>,
    /// The outline to use when drawing the geometry
    pub outline: Outline,
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

#[derive(Debug, Clone)]
pub struct Outline {
    /// The outline thickness to use when drawing the generated image
    ///
    /// The value must not be negative.
    pub thickness: f32,
    /// The color of the outline to draw
    pub color: Rgba,
}

#[derive(Debug, Clone)]
pub enum RenderCamera {
    Camera(Arc<Camera>),
    Query(FileQuery<CameraQuery>),
}

impl RenderCamera {
    pub fn fetch_camera(&self) -> Result<Arc<Camera>, QueryError> {
        use RenderCamera::*;
        match self {
            Camera(cam) => Ok(cam.clone()),
            Query(FileQuery {query, file}) => {
                let mut file = file.lock().expect("bug: file lock was poisoned");
                file.query_camera(query)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum RenderLights {
    Lights(Arc<Vec<Arc<Light>>>),
    Query(FileQuery<LightQuery>),
}

impl RenderLights {
    pub fn fetch_lights(&self) -> Result<Arc<Vec<Arc<Light>>>, QueryError> {
        use RenderLights::*;
        match self {
            Lights(lights) => Ok(lights.clone()),
            Query(FileQuery {query, file}) => {
                let mut file = file.lock().expect("bug: file lock was poisoned");
                file.query_lights(query)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileQuery<Q> {
    pub query: Q,
    pub file: Arc<Mutex<File>>,
}
