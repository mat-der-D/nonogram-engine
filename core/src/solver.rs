mod line;
mod line_solver;

pub use line_solver::LineSolver;

use crate::{
    error::SolveError,
    types::{Problem, SolveResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Filled,
    Empty,
    Unknown,
}

pub type Grid = Vec<Vec<CellState>>;

pub trait Solver {
    fn solve(problem: &Problem) -> Result<SolveResult, SolveError>;
}
