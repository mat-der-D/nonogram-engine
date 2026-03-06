use crate::clue::Clue;
use crate::error::{ClueKind, Error};

/// Represents a complete nonogram puzzle consisting of row and column clues.
///
/// The grid dimensions are derived from the clue list lengths:
/// `height = row_clues.len()` and `width = col_clues.len()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Puzzle {
    row_clues: Vec<Clue>,
    col_clues: Vec<Clue>,
}

impl Puzzle {
    /// Creates a new puzzle from row and column clues.
    ///
    /// # Errors
    /// - `Error::EmptyClueList` if either `row_clues` or `col_clues` is empty.
    /// - `Error::ClueExceedsLength` if any clue's minimum length exceeds
    ///   the corresponding line length.
    pub fn new(row_clues: Vec<Clue>, col_clues: Vec<Clue>) -> Result<Self, Error> {
        if row_clues.is_empty() || col_clues.is_empty() {
            return Err(Error::EmptyClueList);
        }

        let width = col_clues.len();
        let height = row_clues.len();

        for (i, clue) in row_clues.iter().enumerate() {
            if clue.min_length() > width {
                return Err(Error::ClueExceedsLength {
                    kind: ClueKind::Row,
                    line_index: i,
                    clue_min_length: clue.min_length(),
                    line_length: width,
                });
            }
        }

        for (i, clue) in col_clues.iter().enumerate() {
            if clue.min_length() > height {
                return Err(Error::ClueExceedsLength {
                    kind: ClueKind::Col,
                    line_index: i,
                    clue_min_length: clue.min_length(),
                    line_length: height,
                });
            }
        }

        Ok(Self {
            row_clues,
            col_clues,
        })
    }

    /// Returns the number of rows (height).
    pub fn height(&self) -> usize {
        self.row_clues.len()
    }

    /// Returns the number of columns (width).
    pub fn width(&self) -> usize {
        self.col_clues.len()
    }

    /// Returns the row clues.
    pub fn row_clues(&self) -> &[Clue] {
        &self.row_clues
    }

    /// Returns the column clues.
    pub fn col_clues(&self) -> &[Clue] {
        &self.col_clues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ClueKind;

    fn clue(blocks: &[u32]) -> Clue {
        Clue::new(blocks.to_vec()).unwrap()
    }

    #[test]
    fn valid_puzzle() {
        let p = Puzzle::new(vec![clue(&[1]), clue(&[1])], vec![clue(&[1]), clue(&[1])]).unwrap();
        assert_eq!(p.height(), 2);
        assert_eq!(p.width(), 2);
        assert_eq!(p.row_clues().len(), 2);
        assert_eq!(p.col_clues().len(), 2);
    }

    #[test]
    fn empty_row_clues_error() {
        let result = Puzzle::new(vec![], vec![clue(&[1])]);
        assert_eq!(result, Err(Error::EmptyClueList));
    }

    #[test]
    fn empty_col_clues_error() {
        let result = Puzzle::new(vec![clue(&[1])], vec![]);
        assert_eq!(result, Err(Error::EmptyClueList));
    }

    #[test]
    fn row_clue_exceeds_width() {
        // 2 columns, but row clue needs min_length 3
        let result = Puzzle::new(vec![clue(&[3])], vec![clue(&[]), clue(&[])]);
        assert_eq!(
            result,
            Err(Error::ClueExceedsLength {
                kind: ClueKind::Row,
                line_index: 0,
                clue_min_length: 3,
                line_length: 2,
            })
        );
    }

    #[test]
    fn col_clue_exceeds_height() {
        // 1 row, but col clue needs min_length 3
        let result = Puzzle::new(vec![clue(&[])], vec![clue(&[3])]);
        assert_eq!(
            result,
            Err(Error::ClueExceedsLength {
                kind: ClueKind::Col,
                line_index: 0,
                clue_min_length: 3,
                line_length: 1,
            })
        );
    }

    #[test]
    fn puzzle_with_blank_clues() {
        let p = Puzzle::new(vec![clue(&[])], vec![clue(&[])]).unwrap();
        assert_eq!(p.height(), 1);
        assert_eq!(p.width(), 1);
    }
}
