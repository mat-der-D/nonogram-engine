use thiserror::Error;

/// Represents construction-time errors in the nonogram library.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum Error {
    /// A block length of zero was provided when constructing a `Clue`.
    #[error("block at index {block_index} has length zero")]
    InvalidBlockLength {
        /// The index of the zero-length block in the input sequence.
        block_index: usize,
    },
    /// A clue's minimum length exceeds the corresponding line length.
    #[error(
        "{kind} clue at index {line_index} requires minimum length {clue_min_length}, \
         but line length is {line_length}"
    )]
    ClueExceedsLength {
        /// Whether this is a row or column clue.
        kind: ClueKind,
        /// The index of the offending line (row or column).
        line_index: usize,
        /// The minimum length required by the clue.
        clue_min_length: usize,
        /// The actual length of the line.
        line_length: usize,
    },
    /// An empty clue list was provided (0xN or Mx0 puzzle).
    #[error("clue list is empty")]
    EmptyClueList,
}

/// Indicates whether a clue belongs to a row or a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClueKind {
    Row,
    Col,
}

impl std::fmt::Display for ClueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Row => write!(f, "row"),
            Self::Col => write!(f, "column"),
        }
    }
}

/// Represents validation-time errors when checking a solution against a puzzle.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// Grid dimensions do not match puzzle dimensions.
    #[error(
        "dimension mismatch: expected {expected_height}x{expected_width}, \
         got {actual_height}x{actual_width}"
    )]
    DimensionMismatch {
        /// Expected number of rows.
        expected_height: usize,
        /// Expected number of columns.
        expected_width: usize,
        /// Actual number of rows in the grid.
        actual_height: usize,
        /// Actual number of columns in the grid.
        actual_width: usize,
    },
    /// The grid contains one or more `Unknown` cells.
    #[error("grid contains unknown cells")]
    ContainsUnknown,
    /// A row or column does not satisfy its clue.
    #[error("{} {} does not match its clue", if *.is_row { "row" } else { "column" }, .index)]
    ClueMismatch {
        /// `true` for row, `false` for column.
        is_row: bool,
        /// The index of the mismatched row or column.
        index: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_invalid_block_length() {
        let e = Error::InvalidBlockLength { block_index: 2 };
        assert_eq!(e.to_string(), "block at index 2 has length zero");
    }

    #[test]
    fn error_display_clue_exceeds_length_row() {
        let e = Error::ClueExceedsLength {
            kind: ClueKind::Row,
            line_index: 1,
            clue_min_length: 8,
            line_length: 5,
        };
        assert_eq!(
            e.to_string(),
            "row clue at index 1 requires minimum length 8, but line length is 5"
        );
    }

    #[test]
    fn error_display_clue_exceeds_length_col() {
        let e = Error::ClueExceedsLength {
            kind: ClueKind::Col,
            line_index: 0,
            clue_min_length: 3,
            line_length: 1,
        };
        assert_eq!(
            e.to_string(),
            "column clue at index 0 requires minimum length 3, but line length is 1"
        );
    }

    #[test]
    fn error_display_empty_clue_list() {
        let e = Error::EmptyClueList;
        assert_eq!(e.to_string(), "clue list is empty");
    }

    #[test]
    fn validation_error_display_dimension_mismatch() {
        let e = ValidationError::DimensionMismatch {
            expected_height: 5,
            expected_width: 5,
            actual_height: 3,
            actual_width: 4,
        };
        assert_eq!(e.to_string(), "dimension mismatch: expected 5x5, got 3x4");
    }

    #[test]
    fn validation_error_display_contains_unknown() {
        let e = ValidationError::ContainsUnknown;
        assert_eq!(e.to_string(), "grid contains unknown cells");
    }

    #[test]
    fn validation_error_display_clue_mismatch_row() {
        let e = ValidationError::ClueMismatch {
            is_row: true,
            index: 3,
        };
        assert_eq!(e.to_string(), "row 3 does not match its clue");
    }

    #[test]
    fn validation_error_display_clue_mismatch_col() {
        let e = ValidationError::ClueMismatch {
            is_row: false,
            index: 1,
        };
        assert_eq!(e.to_string(), "column 1 does not match its clue");
    }

    #[test]
    fn error_is_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(Error::InvalidBlockLength { block_index: 0 });
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn validation_error_is_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(ValidationError::ContainsUnknown);
        assert!(!e.to_string().is_empty());
    }
}
