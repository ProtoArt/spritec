mod grid;

use thiserror::Error;

use super::{RenderedImage, RenderNode, RenderLayout, Size};
use super::layout::grid::{Grid, GridIter};

#[derive(Debug, Error, PartialEq)]
pub enum LayoutError {
    /// The layout nodes span an area larger than the specified grid
    /// Number of rows/cols required surpasses the amount making up the grid
    #[error("Grid nodes spanned an area larger than the {row} by {col} grid")]
    InsufficientGridSize {row: usize, col: usize},

    /// The node spanned an area that did not (entirely) fit in its designated cell(s)
    #[error("A node did not fit in its designated area on the grid")]
    LayoutNodeDoesNotFit,
}

/// The offset in the image to draw at
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LayoutOffset {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug)]
pub enum LayoutNode {
    RenderedImage(RenderedImage),
    Grid(Grid),
    /// An empty slot, used to create a gap/empty cell in the layout
    Empty {size: Size},
}

impl LayoutNode {
    pub fn from_render_node(node: RenderNode) -> Result<Self, LayoutError> {
        use RenderNode::*;
        match node {
            RenderedImage(image) => Ok(LayoutNode::RenderedImage(image)),
            Layout(RenderLayout::Grid(grid)) => {
                Ok(LayoutNode::Grid(Grid::from_grid_layout(grid)?))
            },
            Empty {size} => Ok(LayoutNode::Empty {size}),
        }
    }

    pub fn size(&self) -> Size {
        use LayoutNode::*;
        match self {
            RenderedImage(image) => image.size,
            Grid(grid) => grid.size(),
            Empty {size} => *size,
        }
    }

    pub fn iter_targets(self) -> LayoutTargetIter {
        use LayoutNode::*;
        match self {
            RenderedImage(_) | Empty {..} => LayoutTargetIter::Single(Some(self)),
            Grid(grid) => LayoutTargetIter::GridIter(grid.iter_targets()),
        }
    }
}

pub enum LayoutTargetIter {
    Single(Option<LayoutNode>),
    GridIter(GridIter),
}

impl Iterator for LayoutTargetIter {
    type Item = (LayoutOffset, LayoutNode);

    fn next(&mut self) -> Option<Self::Item> {
        use LayoutTargetIter::*;
        match self {
            // This default LayoutOffset is specifying that the image takes up all the space
            // given to the image (as opposed to an offset in cells)
            Single(node) => Some((LayoutOffset::default(), node.take()?)),

            GridIter(grid_iter) => grid_iter.next(),
        }
    }
}
