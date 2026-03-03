mod line;
mod line_solver;

pub use line_solver::LineSolver;

use crate::types::{Problem, SolveResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Filled,
    Empty,
    Unknown,
}

pub(crate) type Grid = Vec<Vec<CellState>>;

pub trait Solver {
    fn solve(problem: &Problem) -> SolveResult;
}
