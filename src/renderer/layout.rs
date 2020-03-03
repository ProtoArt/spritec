mod grid;

use super::{RenderedImage, RenderNode, RenderLayout, LayoutType, Size};
use super::layout::grid::{GridLayout, LayoutTargetIter};

#[derive(Debug)]
pub enum LayoutNode {
    RenderedImage(RenderedImage),
    Grid(GridLayout),
    /// An empty slot, used to create a gap/empty cell in the layout
    Empty {size: Size},
}

impl From<RenderNode> for LayoutNode {
    fn from(node: RenderNode) -> Self {
        use RenderNode::*;
        use LayoutType::*;
        match node {
            RenderedImage(image) => LayoutNode::RenderedImage(image),
            Layout(RenderLayout {nodes, layout: Grid {cols}}) => {
                let layout_nodes = nodes.into_iter().map(Into::into).collect();
                LayoutNode::Grid(GridLayout::new(layout_nodes, cols))
            },
            Empty {size} => LayoutNode::Empty {size},
        }
    }
}

impl LayoutNode {
    pub fn size(&self) -> Size {
        use LayoutNode::*;

        match self {
            RenderedImage(image) => image.size,
            Grid(grid) => grid.size(),
            Empty {size} => *size,
        }
    }

    pub fn iter_targets(self) -> LayoutTargetIter {
        LayoutTargetIter {
            node: Some(self),
            current: 0,
        }
    }
}
