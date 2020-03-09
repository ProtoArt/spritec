mod grid;

use thiserror::Error;

use super::{RenderedImage, RenderNode, RenderLayout, Size};
use super::layout::grid::{Grid, GridIter};

#[derive(Debug, Error, PartialEq)]
pub enum LayoutError {
    // Number of rows/cols required surpasses the amount making up the grid
    #[error("Could not fit all images into a {row} by {col} grid")]
    InsufficientGridSize {row: u32, col: u32},

    // Width or height of a layout node is greater than cellsize * row/col span
    #[error("One of the layout nodes is larger than the region allocated for it")]
    LayoutNodeTooLarge,
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
            Layout(layout) => match layout {
                RenderLayout::Grid(grid) => {
                    Ok(LayoutNode::Grid(Grid::from_grid_layout(grid)?))
                },
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
            Single(node) => {
                // This default LayoutOffset is specifying that the image takes up all the space
                // give to the image (as opposed to an offset in cells)
                Some((LayoutOffset::default(), node.take()?))
            },
            GridIter(grid_iter) => {
                grid_iter.next()
            },
        }
    }
}
