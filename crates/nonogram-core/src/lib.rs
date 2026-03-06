mod cell;
mod clue;
mod error;
mod grid;
mod puzzle;
mod solver;

mod backtracker;
mod propagator;
mod validation;

pub use cell::Cell;
pub use clue::Clue;
pub use error::{Error, ValidationError};
pub use grid::Grid;
pub use puzzle::Puzzle;
pub use solver::csp::CspSolver;
pub use solver::probing::ProbingSolver;
pub use solver::{SolveResult, Solver};
pub use validation::validate;

#[cfg(test)]
mod tests {
    use super::*;

    fn clue(blocks: &[u32]) -> Clue {
        Clue::new(blocks.to_vec()).unwrap()
    }

    // --- Task 9.1: dyn Solver trait object tests ---

    #[test]
    fn dyn_solver_csp() {
        let solver: Box<dyn Solver> = Box::new(CspSolver);
        let puzzle = Puzzle::new(vec![clue(&[1])], vec![clue(&[1])]).unwrap();
        let result = solver.solve(&puzzle);
        match result {
            SolveResult::UniqueSolution(grid) => {
                assert_eq!(grid.get(0, 0), Cell::Filled);
            }
            other => panic!("expected UniqueSolution, got {other:?}"),
        }
    }

    #[test]
    fn dyn_solver_probing() {
        let solver: Box<dyn Solver> = Box::new(ProbingSolver);
        let puzzle = Puzzle::new(vec![clue(&[1])], vec![clue(&[1])]).unwrap();
        let result = solver.solve(&puzzle);
        match result {
            SolveResult::UniqueSolution(grid) => {
                assert_eq!(grid.get(0, 0), Cell::Filled);
            }
            other => panic!("expected UniqueSolution, got {other:?}"),
        }
    }

    #[test]
    fn dyn_solver_interchangeable() {
        let puzzle = Puzzle::new(
            vec![clue(&[1]), clue(&[1]), clue(&[5]), clue(&[1]), clue(&[1])],
            vec![clue(&[1]), clue(&[1]), clue(&[5]), clue(&[1]), clue(&[1])],
        )
        .unwrap();

        let solvers: Vec<Box<dyn Solver>> = vec![Box::new(CspSolver), Box::new(ProbingSolver)];

        let results: Vec<SolveResult> = solvers.iter().map(|s| s.solve(&puzzle)).collect();

        // Both should produce UniqueSolution with the same grid.
        match (&results[0], &results[1]) {
            (SolveResult::UniqueSolution(g1), SolveResult::UniqueSolution(g2)) => {
                assert_eq!(g1, g2);
            }
            _ => panic!("expected both to return UniqueSolution"),
        }
    }
}
