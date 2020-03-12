use std::num::NonZeroU32;

use super::{RenderedImage, Size};

#[derive(Debug)]
pub enum RenderNode {
    RenderedImage(RenderedImage),
    Layout(RenderLayout),
    /// An empty slot, used to create a gap/empty cell in the layout
    Empty {size: Size},
}

/// Lays out one or more nodes in the given configuration
#[derive(Debug)]
pub enum RenderLayout {
    Grid(GridLayout),
}

#[derive(Debug)]
pub struct GridLayout {
    /// Number of grid rows
    pub rows: NonZeroU32,
    /// Number of grid columns
    pub cols: NonZeroU32,
    /// Dimensions of each cell in pixels
    pub cell_size: Size,
    /// The nodes to layout
    pub cells: Vec<Vec<GridLayoutCell>>,
}

/// A cell on a grid
#[derive(Debug)]
pub struct GridLayoutCell {
    /// The node to render in this cell
    pub node: RenderNode,
    /// The number of columns spanned for this cell
    pub col_span: NonZeroU32,
    /// The number of rows spanned for this cell
    pub row_span: NonZeroU32,
}

impl GridLayoutCell {
    /// Creates a new cell that spans a single cell
    pub fn single(node: RenderNode) -> Self {
        Self {
            node,
            col_span: unsafe { NonZeroU32::new_unchecked(1) },
            row_span: unsafe { NonZeroU32::new_unchecked(1) },
        }
    }
}
