mod node;
mod render;

pub use node::*;
pub use render::*;

use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::renderer::{ThreadRenderContext, BeginRenderError};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum JobError {
    BeginRenderError(#[from] BeginRenderError),
    DrawError(#[from] glium::DrawError),
    SwapBuffersError(#[from] glium::SwapBuffersError),
    ReadError(#[from] glium::ReadError),
    IOError(#[from] io::Error),
}

#[derive(Debug)]
pub struct RenderJob {
    /// The absolute path to output the generated file
    pub output_path: PathBuf,
    pub root: RenderNode,
}

impl RenderJob {
    pub fn execute(self, ctx: &mut ThreadRenderContext) -> Result<(), JobError> {
        unimplemented!()
    }
}
