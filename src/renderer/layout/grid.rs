use std::num::NonZeroU32;
use std::vec::IntoIter;

use crate::renderer::Size;
use crate::renderer::layout::{LayoutNode, LayoutOffset};
use crate::renderer::render_node::{GridLayout};

use super::LayoutError;

// Availability status of a cell in the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Cell {
    Open,
    Used,
}

#[derive(Debug)]
pub struct GridAvail {
    /// The availability status of each cell stored in a row-wise manner
    cells: Vec<Vec<Cell>>,
    max_row: usize,
    max_col: usize,
    cell_size: Size,
}

impl GridAvail {
    // Creates a new availability grid with all cells set to Open
    fn new(rows: usize, cols: usize, cell_size: Size) -> Self {
        Self {cells: vec![vec![Cell::Open; cols]; rows], max_row: rows, max_col: cols, cell_size}
    }

    /// Searches the given row starting at the given column for a region of the given size.
    /// If a region is found, sets that region to 'Used' and returns the top-left position of the
    /// region as a row index and a column index.
    /// If no region is found, returns an error.
    fn find_avail_region(&mut self, row: usize, start_col: usize, row_span: usize, col_span: usize) -> Result<(usize, usize), LayoutError> {
        if row + row_span > self.max_row || start_col + col_span > self.max_col {
            return Err(LayoutError::InsufficientGridSize {row: self.max_row as u32, col: self.max_col as u32})
        }

        // Subtract col_span from max_col to avoid checking columns we know the node won't fit in
        for col in start_col..=self.max_col - col_span {
            if self.check_avail(row, col, row_span, col_span) {
                self.set_unavail(row, col, row_span, col_span);
                return Ok((row, col));
            }
        }
        Err(LayoutError::InsufficientGridSize {row: self.max_row as u32, col: self.max_col as u32})
    }

    fn check_avail(&self, row: usize, col: usize, row_span: usize, col_span: usize) -> bool {
        for row in &self.cells[row..row+row_span] {
            for cell in &row[col..col+col_span] {
                if *cell == Cell::Used {
                    return false;
                }
            }
        }
        true
    }

    fn set_unavail(&mut self, row: usize, col: usize, row_span: usize, col_span: usize) {
        for row in &mut self.cells[row..row+row_span] {
            for cell in &mut row[col..col+col_span] {
                *cell = Cell::Used;
            }
        }
    }
}

#[derive(Debug)]
pub struct GridCell {
    pub offset: LayoutOffset,
    pub node: LayoutNode,
}

/// A fully-computed grid layout
#[derive(Debug)]
pub struct Grid {
    cells: Vec<GridCell>,
    cell_size: Size,
    rows: NonZeroU32,
    cols: NonZeroU32,
}

impl Grid {
    pub fn from_grid_layout(grid: GridLayout) -> Result<Self, LayoutError> {
        let GridLayout {cell_size, rows, cols, ..} = grid;

        let mut cells = Vec::new();
        // This tracks the available cells in the grid
        let mut free_cells = GridAvail::new(rows.get() as usize, cols.get() as usize, cell_size);

        for (row_index, grid_row) in grid.cells.into_iter().enumerate() {
            for (col_index, grid_layout_cell) in grid_row.into_iter().enumerate() {
                let layout_node = LayoutNode::from_render_node(grid_layout_cell.node)?;
                let row_span = grid_layout_cell.rowspan.get();
                let col_span = grid_layout_cell.colspan.get();
                let cell_width = cell_size.width.get();
                let cell_height = cell_size.height.get();
                let layout_node_width = layout_node.size().width.get();
                let layout_node_height = layout_node.size().height.get();
                // Generate an error if the layout node is larger its specified grid space
                if layout_node_width > (cell_width * col_span)
                    || layout_node_height > (cell_height * row_span) {
                        return Err(LayoutError::LayoutNodeTooLarge);
                }
                // Calculation to find the relative offset to centre the image:
                // Find the centre of the region of the grid that the layout node resides in
                // and subtract half the size of the image from it.
                let x_centre_offset = (cell_width * col_span / 2) - (layout_node_width / 2);
                let y_centre_offset = (cell_height * row_span / 2) - (layout_node_height / 2);
                // Get the row and column index of the top-left corner of an available region
                // that can fit the layout
                let (free_row, free_col) = free_cells.find_avail_region(
                    row_index,
                    col_index,
                    row_span as usize,
                    col_span as usize
                )?;
                let layout_offset = LayoutOffset {
                    x: free_col as u32 * cell_width + x_centre_offset,
                    y: free_row as u32 * cell_height + y_centre_offset,
                };

                cells.push(GridCell {offset: layout_offset, node: layout_node});
            }
        }

        Ok(Self {cells, cell_size, rows, cols})
    }

    /// Returns the total size of the image generated by this layout
    pub fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    /// The total width of the image generated by this layout
    pub fn width(&self) -> NonZeroU32 {
        // Safe because multiplying two non-zero values cannot be zero
        unsafe { NonZeroU32::new_unchecked(self.cell_size.width.get() * self.cols.get()) }
    }

    /// The total height of the image generated by this layout
    pub fn height(&self) -> NonZeroU32 {
        // Safe because multiplying two non-zero values cannot be zero
        unsafe { NonZeroU32::new_unchecked(self.cell_size.height.get() * self.rows.get()) }
    }

    pub fn iter_targets(self) -> GridIter {
        GridIter {cells: self.cells.into_iter()}
    }
}

pub struct GridIter {
    cells: IntoIter<GridCell>,
}

impl Iterator for GridIter {
    type Item = (LayoutOffset, LayoutNode);

    fn next(&mut self) -> Option<Self::Item> {
        self.cells.next().map(|cell| (cell.offset, cell.node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{RenderNode, RenderLayout, GridLayout, GridLayoutCell};

    #[test]
    fn image_too_large() {
        let node = RenderNode::Layout(
            RenderLayout::Grid(
                GridLayout {
                    rows: nz32(1),
                    cols: nz32(3),
                    cell_size: Size {width: nz32(10), height: nz32(10)},
                    cells:
                    vec![
                        vec![
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(25), height: nz32(10)}},
                                colspan: nz32(2),
                                rowspan: nz32(1),
                            },
                        ],
                    ],
                }
            )
        );
        let expected = LayoutError::LayoutNodeTooLarge;

        let actual = LayoutNode::from_render_node(node);
        match actual {
            Ok(_) => panic!("at the disco"),
            Err(err) => assert_eq!(err, expected),
        }
    }


    #[test]
    fn row_span_bound() {
        let node = RenderNode::Layout(
            RenderLayout::Grid(
                GridLayout {
                    rows: nz32(2),
                    cols: nz32(2),
                    cell_size: Size {width: nz32(10), height: nz32(10)},
                    cells:
                    vec![
                        vec![
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(10), height: nz32(30)}},
                                colspan: nz32(1),
                                rowspan: nz32(3),
                            },
                        ],
                        vec![
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(10), height: nz32(10)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(10), height: nz32(10)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                        ],
                    ],
                }
            )
        );
        let expected = LayoutError::InsufficientGridSize {row: 2, col: 2};

        let actual = LayoutNode::from_render_node(node);
        match actual {
            Ok(_) => panic!("at the disco"),
            Err(err) => assert_eq!(err, expected),
        }
    }

    #[test]
    fn col_span_bound() {
        let node = RenderNode::Layout(
            RenderLayout::Grid(
                GridLayout {
                    rows: nz32(2),
                    cols: nz32(2),
                    cell_size: Size {width: nz32(10), height: nz32(10)},
                    cells:
                    vec![
                        vec![
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(10), height: nz32(10)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(10), height: nz32(10)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                        ],
                        vec![
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(10), height: nz32(10)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(20), height: nz32(10)}},
                                colspan: nz32(2),
                                rowspan: nz32(1),
                            },
                        ],
                    ],
                }
            )
        );
        let expected = LayoutError::InsufficientGridSize {row: 2, col: 2};

        let actual = LayoutNode::from_render_node(node);
        match actual {
            Ok(_) => panic!("at the disco"),
            Err(err) => assert_eq!(err, expected),
        }
    }


    // The following test case creates an inner grid of Emptys that is embedded into the bottom row
    // of a 2x2 (40x40 px) grid called node.
    // Offsets refer to the relative top-left position of each layout node (ie. Image, Grid, Empty)
    // Offsets in Node: [(0,0), (20,0), (12, 22)]
    // Offsets in Inner Grid: [(1,0), (5,0), (9,1), (1,9), (8,8), (1,13), (5,13), (9,13), (13,13)]
    //
    //         ┌──────┐              ┌───────────┐
    //         │ Node │              │Inner Grid │
    //     0   └──20──┘    40      0 └─4───8───12┘ 16
    //    0┌───────┬───────┐      0┌┬─┬─┬─┬────────┐
    //     │       │       │       ││ │ │ │ ┌─────┐│
    //     │       │       │       ││ │ │ │ │     ││
    //     │       │       │       ││ │ │ │ │     ││
    //     │       │       │      4││ │ │ │ │     ││
    //     │       │       │       ││ │ │ │ │     ││
    //   20├───────┴───────┤       ││ │ │ │ │     ││
    //     │   ┌───────┐   │       ││ │ │ │ └─────┘│
    //     │   │ Inner │   │      8│└─┘ └─┘┌───────┤
    //     │   │ Grid  │   │       │┌──────┤       │
    //     │   │       │   │       ││      │       │
    //     │   └───────┘   │       ││      │       │
    //   40└───────────────┘     12│└──────┴───────┤
    //                             │┌─┐ ┌─┐ ┌─┐ ┌─┐│
    //                             ││ │ │ │ │ │ │ ││
    //                             │└─┘ └─┘ └─┘ └─┘│
    //                           16└───────────────┘
    #[test]
    fn general_use_grid() {
        let inner_grid = RenderNode::Layout(
            RenderLayout::Grid(
                GridLayout {
                    rows: nz32(4),
                    cols: nz32(4),
                    cell_size: Size {width: nz32(4), height: nz32(4)},
                    cells:
                    vec![
                        vec![
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(2), height: nz32(8)}},
                                colspan: nz32(1),
                                rowspan: nz32(2),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(2), height: nz32(8)}},
                                colspan: nz32(1),
                                rowspan: nz32(2),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(6), height: nz32(6)}},
                                colspan: nz32(2),
                                rowspan: nz32(2),
                            },
                        ],
                        vec![],
                        vec![
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(7), height: nz32(3)}},
                                colspan: nz32(2),
                                rowspan: nz32(1),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(8), height: nz32(4)}},
                                colspan: nz32(2),
                                rowspan: nz32(1),
                            },
                        ],
                        vec![
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(2), height: nz32(2)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(2), height: nz32(2)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(2), height: nz32(2)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(2), height: nz32(2)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                        ],
                    ],
                }
            )
        );
        let node = RenderNode::Layout(
            RenderLayout::Grid(
                GridLayout {
                    rows: nz32(2),
                    cols: nz32(2),
                    cell_size: Size {width: nz32(20), height: nz32(20)},
                    cells:
                    vec![
                        vec![
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(20), height: nz32(20)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                            GridLayoutCell {
                                node: RenderNode::Empty {size: Size {width: nz32(20), height: nz32(20)}},
                                colspan: nz32(1),
                                rowspan: nz32(1),
                            },
                        ],
                        vec![
                            GridLayoutCell {
                                node: inner_grid,
                                colspan: nz32(2),
                                rowspan: nz32(1),
                            },
                        ],
                    ],
                }
            )
        );
        let expected = LayoutNode::Grid(
            Grid {
                cells: vec![
                    GridCell {
                        offset: LayoutOffset {
                            x: 0,
                            y: 0,
                        },
                        node: LayoutNode::Empty {
                            size: Size {
                                width: nz32(20),
                                height: nz32(20),
                            },
                        },
                    },
                    GridCell {
                        offset: LayoutOffset {
                            x: 20,
                            y: 0,
                        },
                        node: LayoutNode::Empty {
                            size: Size {
                                width: nz32(20),
                                height: nz32(20),
                            },
                        },
                    },
                    GridCell {
                        offset: LayoutOffset {
                            x: 12,
                            y: 22,
                        },
                        node: LayoutNode::Grid(
                            Grid {
                                cells: vec![
                                    GridCell {
                                        offset: LayoutOffset {
                                            x: 1,
                                            y: 0,
                                        },
                                        node: LayoutNode::Empty {
                                            size: Size {
                                                width: nz32(2),
                                                height: nz32(8),
                                            },
                                        },
                                    },
                                    GridCell {
                                        offset: LayoutOffset {
                                            x: 5,
                                            y: 0,
                                        },
                                        node: LayoutNode::Empty {
                                            size: Size {
                                                width: nz32(2),
                                                height: nz32(8),
                                            },
                                        },
                                    },
                                    GridCell {
                                        offset: LayoutOffset {
                                            x: 9,
                                            y: 1,
                                        },
                                        node: LayoutNode::Empty {
                                            size: Size {
                                                width: nz32(6),
                                                height: nz32(6),
                                            },
                                        },
                                    },
                                    GridCell {
                                        offset: LayoutOffset {
                                            x: 1,
                                            y: 9,
                                        },
                                        node: LayoutNode::Empty {
                                            size: Size {
                                                width: nz32(7),
                                                height: nz32(3),
                                            },
                                        },
                                    },
                                    GridCell {
                                        offset: LayoutOffset {
                                            x: 8,
                                            y: 8,
                                        },
                                        node: LayoutNode::Empty {
                                            size: Size {
                                                width: nz32(8),
                                                height: nz32(4),
                                            },
                                        },
                                    },
                                    GridCell {
                                        offset: LayoutOffset {
                                            x: 1,
                                            y: 13,
                                        },
                                        node: LayoutNode::Empty {
                                            size: Size {
                                                width: nz32(2),
                                                height: nz32(2),
                                            },
                                        },
                                    },
                                    GridCell {
                                        offset: LayoutOffset {
                                            x: 5,
                                            y: 13,
                                        },
                                        node: LayoutNode::Empty {
                                            size: Size {
                                                width: nz32(2),
                                                height: nz32(2),
                                            },
                                        },
                                    },
                                    GridCell {
                                        offset: LayoutOffset {
                                            x: 9,
                                            y: 13,
                                        },
                                        node: LayoutNode::Empty {
                                            size: Size {
                                                width: nz32(2),
                                                height: nz32(2),
                                            },
                                        },
                                    },
                                    GridCell {
                                        offset: LayoutOffset {
                                            x: 13,
                                            y: 13,
                                        },
                                        node: LayoutNode::Empty {
                                            size: Size {
                                                width: nz32(2),
                                                height: nz32(2),
                                            },
                                        },
                                    },
                                ],
                                cell_size: Size {
                                    width: nz32(4),
                                    height: nz32(4),
                                },
                                rows: nz32(4),
                                cols: nz32(4),
                            },
                        ),
                    },
                ],
                cell_size: Size {
                    width: nz32(20),
                    height: nz32(20),
                },
                rows: nz32(2),
                cols: nz32(2),
            },
        );

        let actual = LayoutNode::from_render_node(node).unwrap();
        assert!(layout_node_eq(&actual, &expected));
    }


    fn layout_node_eq(node1: &LayoutNode, node2: &LayoutNode) -> bool {
        use LayoutNode::*;
        match (node1, node2) {
            (RenderedImage(_), RenderedImage(_)) => panic!("Testing rendered images is unsupported!"),
            (Grid(grid1), Grid(grid2)) => grid_eq(grid1, grid2),
            (Empty {size: size1}, Empty {size: size2}) => size1 == size2,
            _ => false,
        }
    }

    fn grid_eq(grid1: &Grid, grid2: &Grid) -> bool {
        let Grid {cells: cells1, cell_size: cell_size1, rows: rows1, cols: cols1} = grid1;
        let Grid {cells: cells2, cell_size: cell_size2, rows: rows2, cols: cols2} = grid2;
        grid_cells_eq(cells1, cells2) && cell_size1 == cell_size2 && rows1 == rows2 && cols1 == cols2
    }

    fn grid_cells_eq(cells1: &[GridCell], cells2: &[GridCell]) -> bool {
        if cells1.len() != cells2.len() {
            return false;
        }
        cells1.iter().zip(cells2).fold(true, |acc, (c1, c2)| acc && grid_cell_eq(c1, c2))
    }

    fn grid_cell_eq(cell1: &GridCell, cell2: &GridCell) -> bool {
        let GridCell {offset: offset1, node: node1} = cell1;
        let GridCell {offset: offset2, node: node2} = cell2;
        if offset1 != offset2 {
            return false;
        }
        layout_node_eq(node1, node2)
    }

    /// By calling this function, you pinky promise that the value will not be 0
    fn nz32(value: u32) -> NonZeroU32 {
        assert!(value > 0);
        unsafe { NonZeroU32::new_unchecked(value) }
    }
}
