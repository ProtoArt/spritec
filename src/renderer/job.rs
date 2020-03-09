use std::num::NonZeroU32;

use image::RgbaImage;

use super::{
    RenderNode,
    ThreadRenderContext,
    DrawLayoutError,
    layout::LayoutNode,
};

#[derive(Debug)]
pub struct RenderJob {
    /// A scale factor to apply to the generated image. The image is scaled without interpolation.
    /// The value must be greater than zero.
    pub scale: NonZeroU32,
    /// The root node of the tree that describes the image to render
    pub root: RenderNode,
}

impl RenderJob {
    pub fn execute(self, ctx: &mut ThreadRenderContext) -> Result<RgbaImage, DrawLayoutError> {
        let Self {scale, root} = self;

        let layout = LayoutNode::from_render_node(root)?;

        let image = ctx.draw(layout)?;
        let image = ctx.scale(&image, scale)?;

        Ok(image)
    }
}
