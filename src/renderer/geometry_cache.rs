use std::path::Path;
use std::marker::PhantomData;

use glium::{
    backend::glutin::headless::Headless,
};
use glium::glutin::CreationError;
use thiserror::Error;

use crate::loaders::{self, LoaderError};

use super::render_mesh::{RenderModel, RenderMesh};

#[derive(Debug, Error)]
pub enum RenderLoaderError {
    #[error("{0}")]
    CreationError(#[from] CreationError),
    #[error("{0}")]
    LoaderError(#[from] LoaderError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelRef<'a> {
    index: usize,
    /// Used to avoid an unused lifetime error. Allows the ModelRef to be bound to the
    /// lifetime of the render context.
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> ModelRef<'a> {
    fn new(index: usize) -> Self {
        Self {
            index,
            _lifetime: PhantomData,
        }
    }
}

/// This struct manages the geometry stored on the GPU
///
/// The goal is to avoid storing more than necessary and, if possible, remove geometry from the GPU
/// once it is no longer going to be used.
#[derive(Debug, Default)]
pub struct GeometryCache {
    models: Vec<RenderModel>,
}

impl GeometryCache {
    pub fn model(&self, model: ModelRef) -> &RenderModel {
        &self.models[model.index]
    }

    pub fn load_file<'a>(&'a mut self, display: &Headless, path: &Path) -> Result<ModelRef<'a>, RenderLoaderError> {
        let model = loaders::load_file(path)?;
        let render_model = RenderModel {
            meshes: model.meshes.iter().map(|mesh| RenderMesh::new(display, mesh)).collect()?,
        };
        self.models.push(render_model);
        Ok(ModelRef::new(self.models.len() - 1))
    }
}
