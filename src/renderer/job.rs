use std::io;
use std::path::PathBuf;

use thiserror::Error;

use super::{
    RenderNode,
    ThreadRenderContext,
    DrawLayoutError,
    layout::LayoutNode,
};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum JobError {
    DrawLayoutError(#[from] DrawLayoutError),
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
        let Self {output_path, root} = self;

        let layout = LayoutNode::from(root);

        let image = ctx.draw(layout)?;
        image.save(&output_path)?;

        Ok(())
    }
}
