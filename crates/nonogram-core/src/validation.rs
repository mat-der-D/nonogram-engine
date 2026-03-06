use crate::cell::Cell;
use crate::clue::Clue;
use crate::error::ValidationError;
use crate::grid::Grid;
use crate::puzzle::Puzzle;

/// Validates that every row and column in `grid` satisfies the
/// corresponding clue in `puzzle`.
///
/// # Errors
/// - `ValidationError::DimensionMismatch` if grid dimensions differ from puzzle.
/// - `ValidationError::ContainsUnknown` if any cell is `Unknown`.
/// - `ValidationError::ClueMismatch` if a row or column does not match its clue.
pub fn validate(puzzle: &Puzzle, grid: &Grid) -> Result<(), ValidationError> {
    // Check dimensions.
    if grid.height() != puzzle.height() || grid.width() != puzzle.width() {
        return Err(ValidationError::DimensionMismatch {
            expected_height: puzzle.height(),
            expected_width: puzzle.width(),
            actual_height: grid.height(),
            actual_width: grid.width(),
        });
    }

    // Check for Unknown cells.
    for r in 0..grid.height() {
        for c in 0..grid.width() {
            if grid.get(r, c) == Cell::Unknown {
                return Err(ValidationError::ContainsUnknown);
            }
        }
    }

    // Check row clues.
    for (i, clue) in puzzle.row_clues().iter().enumerate() {
        if !line_matches(grid.row(i), clue) {
            return Err(ValidationError::ClueMismatch {
                is_row: true,
                index: i,
            });
        }
    }

    // Check column clues.
    for (i, clue) in puzzle.col_clues().iter().enumerate() {
        let col = grid.col(i);
        if !line_matches(&col, clue) {
            return Err(ValidationError::ClueMismatch {
                is_row: false,
                index: i,
            });
        }
    }

    Ok(())
}

/// Returns `true` if the given line of cells matches the clue.
fn line_matches(line: &[Cell], clue: &Clue) -> bool {
    let blocks: Vec<u32> = extract_blocks(line);
    blocks == clue.blocks()
}

/// Extracts the consecutive runs of `Filled` cells from a line.
fn extract_blocks(line: &[Cell]) -> Vec<u32> {
    let mut blocks = Vec::new();
    let mut count: u32 = 0;
    for &c in line {
        if c == Cell::Filled {
            count += 1;
        } else if count > 0 {
            blocks.push(count);
            count = 0;
        }
    }
    if count > 0 {
        blocks.push(count);
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clue::Clue;

    fn clue(blocks: &[u32]) -> Clue {
        Clue::new(blocks.to_vec()).unwrap()
    }

    fn make_puzzle_and_grid(
        row_clues: Vec<Clue>,
        col_clues: Vec<Clue>,
        cells: &[&[Cell]],
    ) -> (Puzzle, Grid) {
        let h = cells.len();
        let w = if h > 0 { cells[0].len() } else { 0 };
        let puzzle = Puzzle::new(row_clues, col_clues).unwrap();
        let mut grid = Grid::new(h, w);
        for (r, row) in cells.iter().enumerate() {
            for (c, &cell) in row.iter().enumerate() {
                grid.set(r, c, cell);
            }
        }
        (puzzle, grid)
    }

    #[test]
    fn valid_solution() {
        use Cell::*;
        let (puzzle, grid) = make_puzzle_and_grid(
            vec![clue(&[2]), clue(&[])],
            vec![clue(&[1]), clue(&[1])],
            &[&[Filled, Filled], &[Blank, Blank]],
        );
        assert_eq!(validate(&puzzle, &grid), Ok(()));
    }

    #[test]
    fn dimension_mismatch() {
        let puzzle =
            Puzzle::new(vec![clue(&[1]), clue(&[1])], vec![clue(&[1]), clue(&[1])]).unwrap();
        let grid = Grid::new(3, 2);
        assert_eq!(
            validate(&puzzle, &grid),
            Err(ValidationError::DimensionMismatch {
                expected_height: 2,
                expected_width: 2,
                actual_height: 3,
                actual_width: 2,
            })
        );
    }

    #[test]
    fn contains_unknown() {
        let puzzle = Puzzle::new(vec![clue(&[1])], vec![clue(&[1])]).unwrap();
        let grid = Grid::new(1, 1); // All Unknown
        assert_eq!(
            validate(&puzzle, &grid),
            Err(ValidationError::ContainsUnknown)
        );
    }

    #[test]
    fn clue_mismatch_row() {
        use Cell::*;
        let (puzzle, grid) = make_puzzle_and_grid(
            vec![clue(&[2]), clue(&[])],
            vec![clue(&[1]), clue(&[])],
            &[&[Filled, Blank], &[Blank, Blank]],
        );
        // Row 0 has [1] but expects [2].
        assert_eq!(
            validate(&puzzle, &grid),
            Err(ValidationError::ClueMismatch {
                is_row: true,
                index: 0,
            })
        );
    }

    #[test]
    fn clue_mismatch_col() {
        use Cell::*;
        // Rows match, but column 0 has [1] instead of expected [2].
        let (puzzle, grid) = make_puzzle_and_grid(
            vec![clue(&[1]), clue(&[1])],
            vec![clue(&[2]), clue(&[])],
            &[&[Filled, Blank], &[Blank, Filled]],
        );
        assert_eq!(
            validate(&puzzle, &grid),
            Err(ValidationError::ClueMismatch {
                is_row: false,
                index: 0,
            })
        );
    }

    #[test]
    fn dimension_mismatch_checked_before_unknown() {
        // Grid has wrong dimensions AND contains unknown.
        // DimensionMismatch should be returned first.
        let puzzle = Puzzle::new(vec![clue(&[1])], vec![clue(&[1])]).unwrap();
        let grid = Grid::new(2, 1); // Wrong height, contains Unknown
        match validate(&puzzle, &grid) {
            Err(ValidationError::DimensionMismatch { .. }) => {}
            other => panic!("expected DimensionMismatch, got {other:?}"),
        }
    }

    #[test]
    fn unknown_checked_before_clue_mismatch() {
        // Grid correct dimensions but has Unknown.
        let puzzle = Puzzle::new(vec![clue(&[1])], vec![clue(&[1])]).unwrap();
        let grid = Grid::new(1, 1);
        assert_eq!(
            validate(&puzzle, &grid),
            Err(ValidationError::ContainsUnknown)
        );
    }
}
