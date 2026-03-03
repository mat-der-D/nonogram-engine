use crate::{
    error::SolveError,
    types::{Problem, SolveResult},
};

pub trait Solver {
    fn solve(problem: &Problem) -> Result<SolveResult, SolveError>;
}
