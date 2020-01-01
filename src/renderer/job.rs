use std::io;
use std::path::PathBuf;
use std::num::NonZeroU32;

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
    /// A scale factor to apply to the generated image. The image is scaled without interpolation.
    /// The value must be greater than zero.
    pub scale: NonZeroU32,
    /// The root node of the tree that describes the image to render
    pub root: RenderNode,
}

impl RenderJob {
    pub fn execute(self, ctx: &mut ThreadRenderContext) -> Result<(), JobError> {
        let Self {output_path, scale, root} = self;

        let layout = LayoutNode::from(root);

        let image = ctx.draw(layout)?;
        let image = ctx.scale(&image, scale)?;
        image.save(&output_path)?;

        Ok(())
    }
}
