#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    #[error("invalid problem: {0}")]
    InvalidProblem(String),
}
