/// Represents the state of a single cell in a nonogram grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    /// The cell has not yet been determined.
    Unknown,
    /// The cell is filled (colored).
    Filled,
    /// The cell is blank (empty).
    Blank,
}

impl From<bool> for Cell {
    fn from(filled: bool) -> Self {
        if filled { Cell::Filled } else { Cell::Blank }
    }
}
