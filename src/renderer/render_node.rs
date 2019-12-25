use std::num::NonZeroU32;

use super::{Render, Size};

#[derive(Debug)]
pub enum RenderNode {
    Render(Render),
    Layout(RenderLayout),
    /// An empty slot, used to create a gap/empty cell in the layout
    Empty {size: Size},
}

/// Lays out one or more nodes in the given configuration
#[derive(Debug)]
pub struct RenderLayout {
    pub nodes: Vec<RenderNode>,
    pub layout: LayoutType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutType {
  /// All renders are placed in a regular grid with the given number of columns
  Grid { cols: NonZeroU32 },

  //TODO: This is an example of a layout we could have in the future
  // Tightly packs all sprites into an image of width at most the given value. The packing is not
  // guaranteed to be a regular grid.
  //Packed { width: NonZeroU32 },
}
