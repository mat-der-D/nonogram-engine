pub mod csp;
pub mod probing;

use crate::grid::Grid;
use crate::puzzle::Puzzle;

/// The result of solving a nonogram puzzle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SolveResult {
    /// No valid solution exists.
    NoSolution,
    /// Exactly one solution exists.
    UniqueSolution(Grid),
    /// Two or more solutions exist; representative examples are provided.
    ///
    /// The vector always contains at least two elements.
    MultipleSolutions(Vec<Grid>),
}

/// A solver that fully solves a nonogram puzzle.
///
/// All implementations guarantee that `solve` returns a complete result
/// and never returns a grid containing `Unknown` cells as a solution.
pub trait Solver {
    /// Solves the given puzzle and returns the result.
    fn solve(&self, puzzle: &Puzzle) -> SolveResult;
}
