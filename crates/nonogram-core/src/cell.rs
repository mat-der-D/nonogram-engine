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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_are_distinct() {
        assert_ne!(Cell::Unknown, Cell::Filled);
        assert_ne!(Cell::Unknown, Cell::Blank);
        assert_ne!(Cell::Filled, Cell::Blank);
    }

    #[test]
    fn equality() {
        assert_eq!(Cell::Unknown, Cell::Unknown);
        assert_eq!(Cell::Filled, Cell::Filled);
        assert_eq!(Cell::Blank, Cell::Blank);
    }

    #[test]
    fn clone_and_copy() {
        let a = Cell::Filled;
        let b = a; // Copy
        let c = a.clone(); // Clone
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_format() {
        assert_eq!(format!("{:?}", Cell::Unknown), "Unknown");
        assert_eq!(format!("{:?}", Cell::Filled), "Filled");
        assert_eq!(format!("{:?}", Cell::Blank), "Blank");
    }

    #[test]
    fn hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Cell::Unknown);
        set.insert(Cell::Filled);
        set.insert(Cell::Blank);
        assert_eq!(set.len(), 3);
        assert!(set.contains(&Cell::Unknown));
        assert!(set.contains(&Cell::Filled));
        assert!(set.contains(&Cell::Blank));
    }
}
