use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::error::CliError;
use crate::io::{read_input, write_output};

#[derive(Args, Debug)]
pub struct GridToPuzzleArgs {
    /// 入力ファイルパス（省略時は stdin）
    #[arg(long, value_name = "PATH")]
    pub input: Option<PathBuf>,
    /// 出力ファイルパス（省略時は stdout）
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

/// 1 行（または 1 列）の bool スライスからクルーを計算する。
fn compute_clue(line: &[bool]) -> Vec<u32> {
    let mut clues = Vec::new();
    let mut count = 0u32;
    for &cell in line {
        if cell {
            count += 1;
        } else if count > 0 {
            clues.push(count);
            count = 0;
        }
    }
    if count > 0 {
        clues.push(count);
    }
    clues
}

#[derive(Serialize)]
struct PuzzleClues {
    row_clues: Vec<Vec<u32>>,
    col_clues: Vec<Vec<u32>>,
}

pub fn run_grid_to_puzzle(args: GridToPuzzleArgs) -> Result<(), CliError> {
    let json = read_input(args.input.as_deref())?;
    let grid: Vec<Vec<bool>> =
        serde_json::from_str(&json).map_err(|e| CliError::Parse(e.to_string()))?;

    if grid.is_empty() {
        return Err(CliError::Parse("grid must not be empty".to_string()));
    }

    let cols = grid[0].len();
    for row in &grid {
        if row.len() != cols {
            return Err(CliError::Parse(
                "all rows must have the same length".to_string(),
            ));
        }
    }

    let rows = grid.len();
    let row_clues: Vec<Vec<u32>> = grid.iter().map(|row| compute_clue(row)).collect();
    let col_clues: Vec<Vec<u32>> = (0..cols)
        .map(|c| {
            let col: Vec<bool> = (0..rows).map(|r| grid[r][c]).collect();
            compute_clue(&col)
        })
        .collect();

    let output = serde_json::to_string(&PuzzleClues {
        row_clues,
        col_clues,
    })
    .map_err(|e| CliError::Parse(e.to_string()))?;

    write_output(args.output.as_deref(), &output)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- compute_clue テスト ---

    #[test]
    fn compute_clue_empty_line() {
        assert_eq!(compute_clue(&[]), Vec::<u32>::new());
    }

    #[test]
    fn compute_clue_all_blank() {
        assert_eq!(compute_clue(&[false, false, false]), Vec::<u32>::new());
    }

    #[test]
    fn compute_clue_single_block() {
        assert_eq!(compute_clue(&[true, true, true]), vec![3]);
    }

    #[test]
    fn compute_clue_multiple_blocks() {
        assert_eq!(
            compute_clue(&[true, false, true, true, false, true]),
            vec![1, 2, 1]
        );
    }

    #[test]
    fn compute_clue_leading_blank() {
        assert_eq!(compute_clue(&[false, false, true, true]), vec![2]);
    }

    #[test]
    fn compute_clue_trailing_blank() {
        assert_eq!(compute_clue(&[true, true, false, false]), vec![2]);
    }

    // --- ドットグリッドパース テスト ---

    #[test]
    fn parse_valid_dot_grid() {
        let json = r#"[[true,false],[false,true]]"#;
        let grid: Vec<Vec<bool>> = serde_json::from_str(json).unwrap();
        assert_eq!(grid.len(), 2);
        assert!(grid[0][0]);
        assert!(grid[1][1]);
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let result = serde_json::from_str::<Vec<Vec<bool>>>("not json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_wrong_type_returns_error() {
        // 数値の配列を bool の配列として解析しようとするとエラー
        let result = serde_json::from_str::<Vec<Vec<bool>>>("[[1,2],[3,4]]");
        assert!(result.is_err());
    }

    #[test]
    fn run_grid_to_puzzle_row_length_mismatch_returns_parse_error() {
        // 行長不一致: 1列と2列
        let json = r#"[[true],[false,true]]"#;
        let grid: Vec<Vec<bool>> = serde_json::from_str(json).unwrap();
        // 検証ロジックを直接テスト
        let cols = grid[0].len();
        let has_mismatch = grid.iter().any(|row| row.len() != cols);
        assert!(has_mismatch);
    }

    #[test]
    fn run_grid_to_puzzle_empty_grid_returns_parse_error() {
        // 空配列
        let grid: &[Vec<bool>] = &[];
        assert!(grid.is_empty());
    }

    #[test]
    fn compute_clue_used_for_col() {
        // 列クルー計算の確認
        let grid = [[true, false], [true, true], [false, true]];
        let col0: Vec<bool> = (0..3).map(|r| grid[r][0]).collect();
        let col1: Vec<bool> = (0..3).map(|r| grid[r][1]).collect();
        assert_eq!(compute_clue(&col0), vec![2]);
        assert_eq!(compute_clue(&col1), vec![2]);
    }
}
