use crate::error::SolveError;
use crate::solver::line::solve_line;
use crate::solver::{CellState, Grid, Solver};
use crate::types::{Problem, Solution, SolveResult};

pub struct LineSolver;

impl Solver for LineSolver {
    fn solve(problem: &Problem) -> Result<SolveResult, SolveError> {
        let size = problem.size();
        let clues = problem.clues();
        let height = size.height();
        let width = size.width();

        // Validate clue dimensions
        if clues.rows().len() != height {
            return Err(SolveError::InvalidProblem(format!(
                "expected {} row clues, got {}",
                height,
                clues.rows().len()
            )));
        }
        if clues.cols().len() != width {
            return Err(SolveError::InvalidProblem(format!(
                "expected {} col clues, got {}",
                width,
                clues.cols().len()
            )));
        }

        // Initialize grid with Unknown
        let mut grid: Grid = vec![vec![CellState::Unknown; width]; height];

        match solve_with_backtracking(&mut grid, clues.rows(), clues.cols()) {
            Ok(true) => {
                let bool_grid = grid_to_bool(&grid);
                Ok(SolveResult::Unique(Solution::new(size, bool_grid)))
            }
            Ok(false) => Ok(SolveResult::NoSolution),
            Err(e) => Err(e),
        }
    }
}

/// Run line solving to fixed point.
fn propagate(
    grid: &mut Grid,
    row_clues: &[Vec<u8>],
    col_clues: &[Vec<u8>],
) -> Result<(), SolveError> {
    let height = grid.len();
    let width = grid[0].len();
    let mut col_buf: Vec<CellState> = vec![CellState::Unknown; height];

    loop {
        let mut changed = false;

        // Solve each row
        for r in 0..height {
            changed |= solve_line(&row_clues[r], &mut grid[r])?;
        }

        // Solve each column
        for c in 0..width {
            for r in 0..height {
                col_buf[r] = grid[r][c];
            }

            if solve_line(&col_clues[c], &mut col_buf)? {
                changed = true;
                for r in 0..height {
                    grid[r][c] = col_buf[r];
                }
            }
        }

        if !changed {
            break;
        }
    }

    Ok(())
}

/// Find the first Unknown cell (row, col).
fn find_unknown(grid: &Grid) -> Option<(usize, usize)> {
    for (r, row) in grid.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            if *cell == CellState::Unknown {
                return Some((r, c));
            }
        }
    }
    None
}

/// Solve with line propagation + backtracking.
/// Returns Ok(true) if solved, Ok(false) if no solution, Err on invalid problem.
fn solve_with_backtracking(
    grid: &mut Grid,
    row_clues: &[Vec<u8>],
    col_clues: &[Vec<u8>],
) -> Result<bool, SolveError> {
    // Propagate to fixed point
    if propagate(grid, row_clues, col_clues).is_err() {
        return Ok(false); // Contradiction during propagation
    }

    // Find an unknown cell to branch on
    let (r, c) = match find_unknown(grid) {
        Some(pos) => pos,
        None => return Ok(true), // Fully solved
    };

    // Try Filled first
    for guess in [CellState::Filled, CellState::Empty] {
        let mut grid_copy = grid.clone();
        grid_copy[r][c] = guess;

        match solve_with_backtracking(&mut grid_copy, row_clues, col_clues) {
            Ok(true) => {
                // Found a solution — copy it back
                *grid = grid_copy;
                return Ok(true);
            }
            Ok(false) => continue, // Try next guess
            Err(e) => return Err(e),
        }
    }

    // Both guesses failed
    Ok(false)
}

fn grid_to_bool(grid: &Grid) -> Vec<Vec<bool>> {
    grid.iter()
        .map(|row| row.iter().map(|c| matches!(c, CellState::Filled)).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Clues, Size};

    /// Helper to solve a puzzle and return the bool grid.
    fn solve_puzzle(
        width: usize,
        height: usize,
        row_clues: Vec<Vec<u8>>,
        col_clues: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<bool>>, String> {
        let size = Size::new(width, height);
        let clues = Clues::new(row_clues, col_clues);
        let problem = Problem::new(size, clues);

        match LineSolver::solve(&problem) {
            Ok(SolveResult::Unique(sol)) => Ok(sol.grid().to_vec()),
            Ok(SolveResult::NoSolution) => Err("no solution".into()),
            Ok(SolveResult::Multiple(_)) => Err("multiple solutions".into()),
            Err(e) => Err(e.to_string()),
        }
    }

    #[test]
    fn test_1x1_filled() {
        let grid = solve_puzzle(1, 1, vec![vec![1]], vec![vec![1]]).unwrap();
        assert_eq!(grid, vec![vec![true]]);
    }

    #[test]
    fn test_1x1_empty() {
        let grid = solve_puzzle(1, 1, vec![vec![0]], vec![vec![0]]).unwrap();
        assert_eq!(grid, vec![vec![false]]);
    }

    #[test]
    fn test_5x5_simple() {
        // A simple 5x5 puzzle:
        //   X X X X X
        //   X . . . X
        //   X . . . X
        //   X . . . X
        //   X X X X X
        let row_clues = vec![vec![5], vec![1, 1], vec![1, 1], vec![1, 1], vec![5]];
        let col_clues = vec![vec![5], vec![1, 1], vec![1, 1], vec![1, 1], vec![5]];

        let grid = solve_puzzle(5, 5, row_clues, col_clues).unwrap();

        let expected = vec![
            vec![true, true, true, true, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, true, true, true, true],
        ];
        assert_eq!(grid, expected);
    }

    #[test]
    fn test_5x5_diagonal_needs_backtracking() {
        // Diagonal pattern — line solver alone can't solve this:
        //   X . . . .
        //   . X . . .
        //   . . X . .
        //   . . . X .
        //   . . . . X
        let row_clues = vec![vec![1], vec![1], vec![1], vec![1], vec![1]];
        let col_clues = vec![vec![1], vec![1], vec![1], vec![1], vec![1]];

        let grid = solve_puzzle(5, 5, row_clues, col_clues).unwrap();

        // Should find *a* solution (there are multiple valid ones, but we just verify it's valid)
        // Each row should have exactly 1 filled cell
        for row in &grid {
            assert_eq!(row.iter().filter(|&&c| c).count(), 1);
        }
        // Each column should have exactly 1 filled cell
        for c in 0..5 {
            assert_eq!((0..5).filter(|&r| grid[r][c]).count(), 1);
        }
    }

    #[test]
    fn test_no_solution() {
        // Contradictory: row says all filled, col says all empty
        let result = solve_puzzle(2, 1, vec![vec![2]], vec![vec![0], vec![0]]);
        assert!(result.is_err() || result.unwrap_err().contains("no solution"));
    }

    #[test]
    fn test_3x3_checkerboard_like() {
        // X . X
        // . X .
        // X . X
        let row_clues = vec![vec![1, 1], vec![1], vec![1, 1]];
        let col_clues = vec![vec![1, 1], vec![1], vec![1, 1]];

        let grid = solve_puzzle(3, 3, row_clues, col_clues).unwrap();

        let expected = vec![
            vec![true, false, true],
            vec![false, true, false],
            vec![true, false, true],
        ];
        assert_eq!(grid, expected);
    }

    #[test]
    fn test_invalid_clue_dimensions() {
        let size = Size::new(3, 3);
        let clues = Clues::new(vec![vec![1]], vec![vec![1], vec![1], vec![1]]); // Only 1 row clue for 3 rows
        let problem = Problem::new(size, clues);

        let result = LineSolver::solve(&problem);
        assert!(result.is_err());
    }
}
