use crate::backtracker::Backtracker;
use crate::grid::Grid;
use crate::propagator::LinePropagator;
use crate::puzzle::Puzzle;
use crate::solver::{SolveResult, Solver};

/// A complete solver using constraint satisfaction (constraint propagation
/// + backtracking search).
pub struct CspSolver;

impl Solver for CspSolver {
    fn solve(&self, puzzle: &Puzzle) -> SolveResult {
        let mut grid = Grid::new(puzzle.height(), puzzle.width());

        // Phase 1: Constraint propagation.
        match LinePropagator::propagate(&mut grid, puzzle) {
            Ok(_) => {}
            Err(_) => return SolveResult::NoSolution,
        }

        // Phase 2: Check if complete.
        if grid.is_complete() {
            return SolveResult::UniqueSolution(grid);
        }

        // Phase 3: Backtracking search with max_solutions=2.
        let solutions = Backtracker::search(&mut grid, puzzle, 2);
        SolveResult::from_solutions(solutions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::clue::Clue;

    fn clue(blocks: &[u32]) -> Clue {
        Clue::new(blocks.to_vec()).unwrap()
    }

    #[test]
    fn solve_1x1_filled() {
        let puzzle = Puzzle::new(vec![clue(&[1])], vec![clue(&[1])]).unwrap();
        let result = CspSolver.solve(&puzzle);
        match result {
            SolveResult::UniqueSolution(grid) => {
                assert_eq!(grid.get(0, 0), Cell::Filled);
                assert!(grid.is_complete());
            }
            other => panic!("expected UniqueSolution, got {other:?}"),
        }
    }

    #[test]
    fn solve_1x1_blank() {
        let puzzle = Puzzle::new(vec![clue(&[])], vec![clue(&[])]).unwrap();
        let result = CspSolver.solve(&puzzle);
        match result {
            SolveResult::UniqueSolution(grid) => {
                assert_eq!(grid.get(0, 0), Cell::Blank);
            }
            other => panic!("expected UniqueSolution, got {other:?}"),
        }
    }

    #[test]
    fn solve_5x5_cross() {
        let puzzle = Puzzle::new(
            vec![clue(&[1]), clue(&[1]), clue(&[5]), clue(&[1]), clue(&[1])],
            vec![clue(&[1]), clue(&[1]), clue(&[5]), clue(&[1]), clue(&[1])],
        )
        .unwrap();
        let result = CspSolver.solve(&puzzle);
        match result {
            SolveResult::UniqueSolution(grid) => {
                assert!(grid.is_complete());
                // Center is filled.
                assert_eq!(grid.get(2, 2), Cell::Filled);
                // Row 2 is all filled.
                for c in 0..5 {
                    assert_eq!(grid.get(2, c), Cell::Filled);
                }
                // No Unknown cells.
                for r in 0..5 {
                    for c in 0..5 {
                        assert_ne!(grid.get(r, c), Cell::Unknown);
                    }
                }
            }
            other => panic!("expected UniqueSolution, got {other:?}"),
        }
    }

    #[test]
    fn solve_no_solution() {
        // Row wants filled, column wants blank -> contradiction.
        let puzzle = Puzzle::new(vec![clue(&[2])], vec![clue(&[]), clue(&[])]).unwrap();
        let result = CspSolver.solve(&puzzle);
        assert_eq!(result, SolveResult::NoSolution);
    }

    #[test]
    fn solve_multiple_solutions() {
        // 2x2: rows [1],[1]; cols [1],[1] -> 2 solutions
        let puzzle =
            Puzzle::new(vec![clue(&[1]), clue(&[1])], vec![clue(&[1]), clue(&[1])]).unwrap();
        let result = CspSolver.solve(&puzzle);
        match result {
            SolveResult::MultipleSolutions(grids) => {
                assert!(grids.len() >= 2);
                for g in &grids {
                    assert!(g.is_complete());
                    for r in 0..2 {
                        for c in 0..2 {
                            assert_ne!(g.get(r, c), Cell::Unknown);
                        }
                    }
                }
            }
            other => panic!("expected MultipleSolutions, got {other:?}"),
        }
    }

    #[test]
    fn solve_result_contains_no_unknown() {
        let puzzle = Puzzle::new(
            vec![clue(&[1, 1]), clue(&[1])],
            vec![clue(&[1]), clue(&[1]), clue(&[1])],
        )
        .unwrap();
        let result = CspSolver.solve(&puzzle);
        match result {
            SolveResult::UniqueSolution(grid) => {
                for r in 0..grid.height() {
                    for c in 0..grid.width() {
                        assert_ne!(grid.get(r, c), Cell::Unknown);
                    }
                }
            }
            SolveResult::MultipleSolutions(grids) => {
                for g in &grids {
                    for r in 0..g.height() {
                        for c in 0..g.width() {
                            assert_ne!(g.get(r, c), Cell::Unknown);
                        }
                    }
                }
            }
            SolveResult::NoSolution => {}
        }
    }
}
