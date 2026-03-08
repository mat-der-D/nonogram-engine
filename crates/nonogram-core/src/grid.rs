use crate::cell::Cell;

/// Represents an M x N nonogram grid where each cell holds a `Cell` value.
///
/// Cells are stored in a single flat `Vec<Cell>` in row-major order
/// (`cells[row * width + col]`). This gives a single heap allocation per grid,
/// fast `row()` slice access, and cache-friendly sequential access for propagation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    cells: Vec<Cell>,
    height: usize,
    width: usize,
}

impl Grid {
    /// Creates a new grid with all cells set to `Unknown`.
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            cells: vec![Cell::Unknown; height * width],
            height,
            width,
        }
    }

    /// Returns the cell value at the given position.
    ///
    /// # Panics
    /// Panics if `row >= height` or `col >= width`.
    pub fn get(&self, row: usize, col: usize) -> Cell {
        assert!(row < self.height, "row index {row} out of bounds (height={})", self.height);
        assert!(col < self.width, "col index {col} out of bounds (width={})", self.width);
        self.cells[row * self.width + col]
    }

    /// Sets the cell value at the given position.
    ///
    /// # Panics
    /// Panics if `row >= height` or `col >= width`.
    pub fn set(&mut self, row: usize, col: usize, value: Cell) {
        assert!(row < self.height, "row index {row} out of bounds (height={})", self.height);
        assert!(col < self.width, "col index {col} out of bounds (width={})", self.width);
        self.cells[row * self.width + col] = value;
    }

    /// Returns the number of rows.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the number of columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns a reference to the specified row as a slice.
    ///
    /// # Panics
    /// Panics if `index >= height`.
    pub fn row(&self, index: usize) -> &[Cell] {
        let start = index * self.width;
        &self.cells[start..start + self.width]
    }

    /// Returns a copy of the specified column as a `Vec<Cell>`.
    ///
    /// # Panics
    /// Panics if `index >= width`.
    pub fn col(&self, index: usize) -> Vec<Cell> {
        (0..self.height)
            .map(|r| self.cells[r * self.width + index])
            .collect()
    }

    /// Fills `buf` with the values of the specified column.
    ///
    /// Avoids allocation compared to [`col`]. `buf` must have length `>= height`.
    ///
    /// # Panics
    /// Panics if `index >= width` or `buf.len() < height`.
    pub(crate) fn fill_col(&self, index: usize, buf: &mut [Cell]) {
        for (r, cell) in buf.iter_mut().enumerate().take(self.height) {
            *cell = self.cells[r * self.width + index];
        }
    }

    /// Returns `true` if no cell is `Unknown`.
    pub fn is_complete(&self) -> bool {
        self.cells.iter().all(|&c| c != Cell::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_all_unknown() {
        let g = Grid::new(3, 4);
        assert_eq!(g.height(), 3);
        assert_eq!(g.width(), 4);
        for r in 0..3 {
            for c in 0..4 {
                assert_eq!(g.get(r, c), Cell::Unknown);
            }
        }
    }

    #[test]
    fn set_and_get() {
        let mut g = Grid::new(2, 2);
        g.set(0, 1, Cell::Filled);
        g.set(1, 0, Cell::Blank);
        assert_eq!(g.get(0, 1), Cell::Filled);
        assert_eq!(g.get(1, 0), Cell::Blank);
        assert_eq!(g.get(0, 0), Cell::Unknown);
    }

    #[test]
    fn row_access() {
        let mut g = Grid::new(2, 3);
        g.set(0, 0, Cell::Filled);
        g.set(0, 1, Cell::Blank);
        g.set(0, 2, Cell::Filled);
        assert_eq!(g.row(0), &[Cell::Filled, Cell::Blank, Cell::Filled]);
    }

    #[test]
    fn col_access() {
        let mut g = Grid::new(3, 2);
        g.set(0, 0, Cell::Filled);
        g.set(1, 0, Cell::Blank);
        g.set(2, 0, Cell::Filled);
        assert_eq!(g.col(0), vec![Cell::Filled, Cell::Blank, Cell::Filled]);
    }

    #[test]
    fn is_complete_false_when_unknown() {
        let g = Grid::new(2, 2);
        assert!(!g.is_complete());
    }

    #[test]
    fn is_complete_true_when_all_determined() {
        let mut g = Grid::new(2, 2);
        g.set(0, 0, Cell::Filled);
        g.set(0, 1, Cell::Blank);
        g.set(1, 0, Cell::Blank);
        g.set(1, 1, Cell::Filled);
        assert!(g.is_complete());
    }

    #[test]
    #[should_panic]
    fn get_out_of_bounds_row() {
        let g = Grid::new(2, 2);
        g.get(2, 0);
    }

    #[test]
    #[should_panic]
    fn get_out_of_bounds_col() {
        let g = Grid::new(2, 2);
        g.get(0, 2);
    }

    #[test]
    fn clone_is_independent() {
        let mut g = Grid::new(2, 2);
        g.set(0, 0, Cell::Filled);
        let mut g2 = g.clone();
        g2.set(0, 0, Cell::Blank);
        assert_eq!(g.get(0, 0), Cell::Filled);
        assert_eq!(g2.get(0, 0), Cell::Blank);
    }
}
