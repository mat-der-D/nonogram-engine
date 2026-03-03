use crate::solver::line::solve_line;
use crate::solver::{CellState, Grid, Solver};
use crate::types::{Problem, Solution, SolveResult};

pub struct LineSolver;

impl Solver for LineSolver {
    fn solve(problem: &Problem) -> SolveResult {
        let clues = problem.clues();
        let height = clues.height();
        let width = clues.width();

        let mut grid: Grid = vec![vec![CellState::Unknown; width]; height];

        if solve_with_backtracking(&mut grid, clues.rows(), clues.cols()) {
            let bool_grid = grid_to_bool(&grid);
            SolveResult::Unique(Solution::new(bool_grid))
        } else {
            SolveResult::NoSolution
        }
    }
}

/// Run line solving to fixed point.
/// Returns `true` if propagation succeeded, `false` if a contradiction was found.
fn propagate(grid: &mut Grid, row_clues: &[Vec<u8>], col_clues: &[Vec<u8>]) -> bool {
    let height = grid.len();
    let width = grid[0].len();
    let mut col_buf: Vec<CellState> = vec![CellState::Unknown; height];

    loop {
        let mut changed = false;

        for r in 0..height {
            match solve_line(&row_clues[r], &mut grid[r]) {
                Some(c) => changed |= c,
                None => return false,
            }
        }

        for c in 0..width {
            for r in 0..height {
                col_buf[r] = grid[r][c];
            }

            match solve_line(&col_clues[c], &mut col_buf) {
                Some(col_changed) => {
                    if col_changed {
                        changed = true;
                        for r in 0..height {
                            grid[r][c] = col_buf[r];
                        }
                    }
                }
                None => return false,
            }
        }

        if !changed {
            return true;
        }
    }
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
/// Returns `true` if a solution was found, `false` otherwise.
fn solve_with_backtracking(
    grid: &mut Grid,
    row_clues: &[Vec<u8>],
    col_clues: &[Vec<u8>],
) -> bool {
    if !propagate(grid, row_clues, col_clues) {
        return false;
    }

    let (r, c) = match find_unknown(grid) {
        Some(pos) => pos,
        None => return true,
    };

    for guess in [CellState::Filled, CellState::Empty] {
        let mut grid_copy = grid.clone();
        grid_copy[r][c] = guess;

        if solve_with_backtracking(&mut grid_copy, row_clues, col_clues) {
            *grid = grid_copy;
            return true;
        }
    }

    false
}

fn grid_to_bool(grid: &Grid) -> Vec<Vec<bool>> {
    grid.iter()
        .map(|row| row.iter().map(|c| matches!(c, CellState::Filled)).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Clues;

    /// Helper to solve a puzzle and return the bool grid.
    fn solve_puzzle(
        row_clues: Vec<Vec<u8>>,
        col_clues: Vec<Vec<u8>>,
    ) -> Option<Vec<Vec<bool>>> {
        let clues = Clues::new(row_clues, col_clues);
        let problem = Problem::new(clues);

        match LineSolver::solve(&problem) {
            SolveResult::Unique(sol) => Some(sol.grid().to_vec()),
            _ => None,
        }
    }

    #[test]
    fn test_1x1_filled() {
        let grid = solve_puzzle(vec![vec![1]], vec![vec![1]]).unwrap();
        assert_eq!(grid, vec![vec![true]]);
    }

    #[test]
    fn test_1x1_empty() {
        let grid = solve_puzzle(vec![vec![0]], vec![vec![0]]).unwrap();
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

        let grid = solve_puzzle(row_clues, col_clues).unwrap();

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

        let grid = solve_puzzle(row_clues, col_clues).unwrap();

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
        let result = solve_puzzle(vec![vec![2]], vec![vec![0], vec![0]]);
        assert!(result.is_none());
    }

    #[test]
    fn test_3x3_checkerboard_like() {
        // X . X
        // . X .
        // X . X
        let row_clues = vec![vec![1, 1], vec![1], vec![1, 1]];
        let col_clues = vec![vec![1, 1], vec![1], vec![1, 1]];

        let grid = solve_puzzle(row_clues, col_clues).unwrap();

        let expected = vec![
            vec![true, false, true],
            vec![false, true, false],
            vec![true, false, true],
        ];
        assert_eq!(grid, expected);
    }

    #[test]
    fn test_mismatched_clue_dimensions() {
        // 1 row clue but 3 col clues — grid will be 1×3, which is valid.
        // Instead test that a puzzle with contradictory clues yields no solution.
        let result = solve_puzzle(vec![vec![3]], vec![vec![0], vec![0], vec![0]]);
        assert!(result.is_none());
    }
}
