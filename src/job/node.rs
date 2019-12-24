use super::Render;

#[derive(Debug)]
pub enum RenderNode {
    Render(Render),
    Layout(RenderLayout),
}

/// Lays out one or more nodes in the given configuration
#[derive(Debug)]
pub struct RenderLayout {
    pub nodes: Vec<RenderNode>,
    pub layout: Layout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layout {
  /// All renders are placed in a regular grid with the given number of columns
  Grid { cols: usize },

  //TODO: This is an example of a layout we could have in the future
  // Tightly packs all sprites into an image of width at most the given value. The packing is not
  // guaranteed to be a regular grid.
  //Packed { width: u32 },
}
