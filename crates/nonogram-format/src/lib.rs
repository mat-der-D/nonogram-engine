use nonogram_core::{Cell, Clue, Grid, Puzzle, SolveResult};
use serde::{Deserialize, Serialize};

/// nonogram-format における変換エラー。
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// JSON のパースまたはフィールド検証に失敗した（欠落フィールド・型不一致・範囲外を含む）。
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// 有効な JSON だが Puzzle 構築に失敗した（EmptyClueList など）。
    #[error("invalid puzzle: {0}")]
    InvalidPuzzle(#[from] nonogram_core::Error),

    /// グリッドに Cell::Unknown が含まれており JSON にシリアライズできない。
    #[error("grid contains unknown cells")]
    UnknownCell,

    /// cells の行数または列数が rows/cols と一致しない。
    #[error("shape mismatch: expected {rows}x{cols}, got {actual_rows}x{actual_cols}")]
    ShapeMismatch {
        rows: usize,
        cols: usize,
        actual_rows: usize,
        actual_cols: usize,
    },
}

#[derive(Serialize, Deserialize)]
struct GridDto {
    rows: usize,
    cols: usize,
    cells: Vec<Vec<bool>>,
}

fn grid_to_bool_matrix(grid: &Grid) -> Result<Vec<Vec<bool>>, FormatError> {
    (0..grid.height())
        .map(|r| {
            grid.row(r)
                .iter()
                .map(|&cell| match cell {
                    Cell::Filled => Ok(true),
                    Cell::Blank => Ok(false),
                    Cell::Unknown => Err(FormatError::UnknownCell),
                })
                .collect::<Result<Vec<bool>, _>>()
        })
        .collect()
}

/// Grid を JSON 文字列にシリアライズする。
///
/// # Errors
/// - `FormatError::UnknownCell` — グリッドに `Cell::Unknown` が含まれる場合
/// - `FormatError::Json` — serde_json シリアライズ失敗
pub fn grid_to_json(grid: &Grid) -> Result<String, FormatError> {
    let dto = GridDto { rows: grid.height(), cols: grid.width(), cells: grid_to_bool_matrix(grid)? };
    Ok(serde_json::to_string(&dto)?)
}

/// JSON 文字列から Grid をデシリアライズする。
///
/// # Errors
/// - `FormatError::Json` — 不正な JSON またはフィールド欠落
/// - `FormatError::ShapeMismatch` — rows/cols と cells の次元が一致しない
pub fn json_to_grid(json: &str) -> Result<Grid, FormatError> {
    let dto: GridDto = serde_json::from_str(json)?;
    let actual_rows = dto.cells.len();
    let actual_cols = dto.cells.first().map_or(0, |r| r.len());
    let bad_col = dto.cells.iter().map(|r| r.len()).find(|&l| l != dto.cols);
    if actual_rows != dto.rows || bad_col.is_some() {
        return Err(FormatError::ShapeMismatch {
            rows: dto.rows,
            cols: dto.cols,
            actual_rows,
            actual_cols: bad_col.unwrap_or(actual_cols),
        });
    }
    let mut grid = Grid::new(dto.rows, dto.cols);
    for (r, row) in dto.cells.iter().enumerate() {
        for (c, &filled) in row.iter().enumerate() {
            grid.set(r, c, Cell::from(filled));
        }
    }
    Ok(grid)
}

#[derive(Serialize, Deserialize)]
struct PuzzleDto {
    row_clues: Vec<Vec<u32>>,
    col_clues: Vec<Vec<u32>>,
}

#[derive(Serialize)]
struct SolutionDto {
    status: &'static str,
    solutions: Vec<Vec<Vec<bool>>>,
}

fn parse_clues(raw: Vec<Vec<u32>>) -> Result<Vec<Clue>, FormatError> {
    raw.into_iter()
        .map(|blocks| Clue::new(blocks).map_err(FormatError::InvalidPuzzle))
        .collect()
}

/// JSON 文字列から Puzzle を生成する。
///
/// # Errors
/// - `FormatError::Json` — 不正な JSON、`row_clues`/`col_clues` フィールド欠落、または `u32` 範囲超過
/// - `FormatError::InvalidPuzzle` — `Puzzle` 構築エラー（`Error::EmptyClueList` など）
pub fn puzzle_from_json(json: &str) -> Result<Puzzle, FormatError> {
    let dto: PuzzleDto = serde_json::from_str(json)?;
    let row_clues = parse_clues(dto.row_clues)?;
    let col_clues = parse_clues(dto.col_clues)?;
    let puzzle = Puzzle::new(row_clues, col_clues)?;
    Ok(puzzle)
}

/// SolveResult を JSON 文字列にシリアライズする。
///
/// # Errors
/// - `FormatError::UnknownCell` — グリッドに `Cell::Unknown` が含まれる場合
/// - `FormatError::Json` — serde_json シリアライズ失敗（実運用上は発生しない）
pub fn result_to_json(result: &SolveResult) -> Result<String, FormatError> {
    let dto = match result {
        SolveResult::NoSolution => SolutionDto {
            status: "none",
            solutions: vec![],
        },
        SolveResult::UniqueSolution(grid) => SolutionDto {
            status: "unique",
            solutions: vec![grid_to_bool_matrix(grid)?],
        },
        SolveResult::MultipleSolutions(grids) => SolutionDto {
            status: "multiple",
            solutions: grids
                .iter()
                .map(grid_to_bool_matrix)
                .collect::<Result<_, _>>()?,
        },
    };

    Ok(serde_json::to_string(&dto)?)
}

/// 行数 `rows`・列数 `cols` の空問題テンプレート JSON 文字列を生成する。
///
/// `row_clues` に長さ `rows` の配列、`col_clues` に長さ `cols` の配列を持つ JSON を返す。
/// 各要素は空配列 `[]` である。
pub fn generate_template(rows: usize, cols: usize) -> String {
    let dto = PuzzleDto {
        row_clues: vec![vec![]; rows],
        col_clues: vec![vec![]; cols],
    };
    serde_json::to_string(&dto).expect("PuzzleDto with empty clues always serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use nonogram_core::{Grid, SolveResult};

    // --- Task 1.2: grid_to_json / json_to_grid テスト ---

    #[test]
    fn grid_round_trip() {
        let mut grid = Grid::new(2, 3);
        grid.set(0, 0, Cell::Filled);
        grid.set(0, 1, Cell::Blank);
        grid.set(0, 2, Cell::Filled);
        grid.set(1, 0, Cell::Blank);
        grid.set(1, 1, Cell::Filled);
        grid.set(1, 2, Cell::Blank);
        let json = grid_to_json(&grid).unwrap();
        let restored = json_to_grid(&json).unwrap();
        assert_eq!(restored.height(), 2);
        assert_eq!(restored.width(), 3);
        assert_eq!(restored.get(0, 0), Cell::Filled);
        assert_eq!(restored.get(0, 1), Cell::Blank);
        assert_eq!(restored.get(1, 1), Cell::Filled);
    }

    #[test]
    fn json_to_grid_shape_mismatch_rows() {
        // rows=2 but cells has 3 rows
        let json = r#"{"rows":2,"cols":2,"cells":[[true,false],[false,true],[true,true]]}"#;
        let err = json_to_grid(json).unwrap_err();
        assert!(matches!(err, FormatError::ShapeMismatch { rows: 2, cols: 2, actual_rows: 3, actual_cols: 2 }));
    }

    #[test]
    fn json_to_grid_shape_mismatch_cols() {
        // cols=2 but first row has 3 cells
        let json = r#"{"rows":1,"cols":2,"cells":[[true,false,true]]}"#;
        let err = json_to_grid(json).unwrap_err();
        assert!(matches!(err, FormatError::ShapeMismatch { rows: 1, cols: 2, actual_rows: 1, actual_cols: 3 }));
    }

    #[test]
    fn json_to_grid_invalid_json_returns_json_error() {
        let err = json_to_grid("not valid json").unwrap_err();
        assert!(matches!(err, FormatError::Json(_)));
    }

    #[test]
    fn json_to_grid_missing_field_returns_json_error() {
        // cols フィールド欠落
        let json = r#"{"rows":1,"cells":[[true]]}"#;
        let err = json_to_grid(json).unwrap_err();
        assert!(matches!(err, FormatError::Json(_)));
    }

    #[test]
    fn grid_to_json_unknown_cell_returns_error() {
        let grid = Grid::new(1, 1); // Cell::Unknown のまま
        let err = grid_to_json(&grid).unwrap_err();
        assert!(matches!(err, FormatError::UnknownCell));
    }

    // --- Task 2: puzzle_from_json 正常系テスト ---

    #[test]
    fn puzzle_from_json_valid() {
        // row 0: [1,2] min_length=4, row 1: [3] min_length=3; width=4
        // col 0-3: [1],[2],[1],[1] min_lengths=1,2,1,1; height=2
        let json = r#"{"row_clues":[[1,2],[3]],"col_clues":[[1],[2],[1],[1]]}"#;
        let puzzle = puzzle_from_json(json).unwrap();
        assert_eq!(puzzle.height(), 2);
        assert_eq!(puzzle.width(), 4);
        assert_eq!(puzzle.row_clues()[0].blocks(), &[1, 2]);
        assert_eq!(puzzle.row_clues()[1].blocks(), &[3]);
        assert_eq!(puzzle.col_clues()[0].blocks(), &[1]);
        assert_eq!(puzzle.col_clues()[1].blocks(), &[2]);
    }

    #[test]
    fn puzzle_from_json_empty_array_clue() {
        let json = r#"{"row_clues":[[]],"col_clues":[[]]}"#;
        let puzzle = puzzle_from_json(json).unwrap();
        assert!(puzzle.row_clues()[0].is_empty());
        assert!(puzzle.col_clues()[0].is_empty());
    }

    #[test]
    fn puzzle_from_json_multiple_blocks() {
        // row: [1,1,1] min_length=5; need width>=5
        let json = r#"{"row_clues":[[1,1,1]],"col_clues":[[1],[1],[1],[1],[1]]}"#;
        let puzzle = puzzle_from_json(json).unwrap();
        assert_eq!(puzzle.row_clues()[0].blocks(), &[1, 1, 1]);
        assert_eq!(puzzle.width(), 5);
    }

    // --- Task 2: puzzle_from_json 異常系テスト ---

    #[test]
    fn puzzle_from_json_missing_row_clues() {
        let json = r#"{"col_clues":[[1]]}"#;
        let err = puzzle_from_json(json).unwrap_err();
        assert!(matches!(err, FormatError::Json(_)));
    }

    #[test]
    fn puzzle_from_json_missing_col_clues() {
        let json = r#"{"row_clues":[[1]]}"#;
        let err = puzzle_from_json(json).unwrap_err();
        assert!(matches!(err, FormatError::Json(_)));
    }

    #[test]
    fn puzzle_from_json_u32_overflow() {
        // u32::MAX + 1 = 4294967296
        let json = r#"{"row_clues":[[4294967296]],"col_clues":[[1]]}"#;
        let err = puzzle_from_json(json).unwrap_err();
        assert!(matches!(err, FormatError::Json(_)));
    }

    // --- Task 3: result_to_json 正常系テスト ---

    #[test]
    fn result_to_json_no_solution() {
        let result = SolveResult::NoSolution;
        let json = result_to_json(&result).unwrap();
        assert_eq!(json, r#"{"status":"none","solutions":[]}"#);
    }

    #[test]
    fn result_to_json_unique_solution() {
        let mut grid = Grid::new(2, 2);
        grid.set(0, 0, Cell::Filled);
        grid.set(0, 1, Cell::Blank);
        grid.set(1, 0, Cell::Blank);
        grid.set(1, 1, Cell::Filled);
        let result = SolveResult::UniqueSolution(grid);
        let json = result_to_json(&result).unwrap();
        assert_eq!(
            json,
            r#"{"status":"unique","solutions":[[[true,false],[false,true]]]}"#
        );
    }

    #[test]
    fn result_to_json_multiple_solutions() {
        let mut g1 = Grid::new(1, 2);
        g1.set(0, 0, Cell::Filled);
        g1.set(0, 1, Cell::Blank);
        let mut g2 = Grid::new(1, 2);
        g2.set(0, 0, Cell::Blank);
        g2.set(0, 1, Cell::Filled);
        let result = SolveResult::MultipleSolutions(vec![g1, g2]);
        let json = result_to_json(&result).unwrap();
        assert_eq!(
            json,
            r#"{"status":"multiple","solutions":[[[true,false]],[[false,true]]]}"#
        );
    }

    #[test]
    fn result_to_json_row_major_order() {
        // solutions[0][row][col]: 先頭行先頭列が [0][0][0]
        let mut grid = Grid::new(2, 3);
        grid.set(0, 0, Cell::Filled);
        grid.set(0, 1, Cell::Filled);
        grid.set(0, 2, Cell::Blank);
        grid.set(1, 0, Cell::Blank);
        grid.set(1, 1, Cell::Blank);
        grid.set(1, 2, Cell::Filled);
        let result = SolveResult::UniqueSolution(grid);
        let json = result_to_json(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["solutions"][0][0][0], true);
        assert_eq!(v["solutions"][0][0][1], true);
        assert_eq!(v["solutions"][0][0][2], false);
        assert_eq!(v["solutions"][0][1][0], false);
        assert_eq!(v["solutions"][0][1][2], true);
    }

    // --- Task 3: result_to_json 異常系テスト ---

    #[test]
    fn result_to_json_unknown_cell_error() {
        let grid = Grid::new(1, 1); // Cell::Unknown のまま
        let result = SolveResult::UniqueSolution(grid);
        let err = result_to_json(&result).unwrap_err();
        assert!(matches!(err, FormatError::UnknownCell));
    }

    // --- Task 4: generate_template テスト ---

    #[test]
    fn generate_template_clue_lengths() {
        let json = generate_template(3, 2);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["row_clues"].as_array().unwrap().len(), 3);
        assert_eq!(v["col_clues"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn generate_template_round_trip() {
        let json = generate_template(3, 2);
        let puzzle = puzzle_from_json(&json).unwrap();
        assert_eq!(puzzle.height(), 3);
        assert_eq!(puzzle.width(), 2);
    }

    #[test]
    fn generate_template_structure() {
        let json = generate_template(5, 10);
        let expected = r#"{"row_clues":[[],[],[],[],[]],"col_clues":[[],[],[],[],[],[],[],[],[],[]]}"#;
        assert_eq!(json, expected);
    }
}
