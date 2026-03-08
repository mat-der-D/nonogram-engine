use std::path::PathBuf;

use clap::{Args, ValueEnum};
use nonogram_core::{CspSolver, ProbingSolver, Solver};
use nonogram_format::{puzzle_from_json, result_to_json};

use crate::error::CliError;
use crate::io::{read_input, write_output};

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum SolverKind {
    Csp,
    Probing,
}

#[derive(Args, Debug)]
pub struct SolveArgs {
    /// 入力ファイルパス（省略時は stdin）
    #[arg(long, value_name = "PATH")]
    pub input: Option<PathBuf>,
    /// 出力ファイルパス（省略時は stdout）
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// 使用するソルバ
    #[arg(long, value_enum, default_value = "csp")]
    pub solver: SolverKind,
}

pub fn run_solve(args: SolveArgs) -> Result<(), CliError> {
    let json = read_input(args.input.as_deref())?;
    let puzzle = puzzle_from_json(&json).map_err(|e| CliError::Parse(e.to_string()))?;

    let solver: Box<dyn Solver> = match args.solver {
        SolverKind::Csp => Box::new(CspSolver),
        SolverKind::Probing => Box::new(ProbingSolver),
    };

    let result = solver.solve(&puzzle);
    let output = result_to_json(&result).map_err(|e| CliError::Parse(e.to_string()))?;

    write_output(args.output.as_deref(), &output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_puzzle_json() -> &'static str {
        // 1x1 パズル: row=[1], col=[1] → 唯一解
        r#"{"row_clues":[[1]],"col_clues":[[1]]}"#
    }

    fn no_solution_puzzle_json() -> &'static str {
        // row=[1], col=[] → 解なし（row clue が 1 だが col は空）
        r#"{"row_clues":[[1]],"col_clues":[[]]}"#
    }

    #[test]
    fn run_solve_unique_solution_outputs_json() {
        let json = unique_puzzle_json();
        let puzzle = puzzle_from_json(json).unwrap();
        let result = CspSolver.solve(&puzzle);
        let output = result_to_json(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["status"], "unique");
        assert_eq!(v["solutions"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn run_solve_no_solution_outputs_none_status() {
        let json = no_solution_puzzle_json();
        let puzzle = puzzle_from_json(json).unwrap();
        let result = CspSolver.solve(&puzzle);
        let output = result_to_json(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(v["status"], "none");
        assert_eq!(v["solutions"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn run_solve_parse_error_on_invalid_json() {
        let result = puzzle_from_json("not json");
        assert!(result.is_err());
        let cli_err = CliError::Parse(result.unwrap_err().to_string());
        assert!(matches!(cli_err, CliError::Parse(_)));
    }

    #[test]
    fn run_solve_probing_solver_produces_same_result() {
        let json = unique_puzzle_json();
        let puzzle = puzzle_from_json(json).unwrap();
        let r1 = CspSolver.solve(&puzzle);
        let r2 = ProbingSolver.solve(&puzzle);
        let j1 = result_to_json(&r1).unwrap();
        let j2 = result_to_json(&r2).unwrap();
        let v1: serde_json::Value = serde_json::from_str(&j1).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&j2).unwrap();
        assert_eq!(v1["status"], v2["status"]);
    }
}
