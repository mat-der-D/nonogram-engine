use nonogram_core::{ProbingSolver, Solver};
use nonogram_format::{grid_to_json, puzzle_from_json, result_to_json};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Error response returned to JavaScript when solve fails.
#[derive(Serialize)]
struct ErrorResponseDto {
    status: &'static str,
    message: String,
}

fn error_json(message: String) -> String {
    let dto = ErrorResponseDto {
        status: "error",
        message,
    };
    serde_json::to_string(&dto).expect("ErrorResponseDto always serializes")
}

/// Success response for image_to_grid.
#[derive(Serialize)]
struct OkGridResponseDto {
    status: &'static str,
    grid: serde_json::Value,
}

/// Convert an image to a nonogram grid.
///
/// # Parameters
/// - `image_bytes`: Raw bytes of a PNG/JPEG/WebP/GIF image
/// - `grid_width`: Output grid width (5–50)
/// - `grid_height`: Output grid height (5–50)
/// - `smooth_strength`: Gaussian blur sigma (0 to skip)
/// - `threshold`: Binarisation threshold (0–255)
/// - `edge_strength`: Edge merge strength (0–1)
/// - `noise_removal`: Minimum connected-component size to keep (0 to disable)
///
/// # Returns
/// On success: `{"status":"ok","grid":{"rows":N,"cols":M,"cells":[[...],...]}}`
/// On failure: `{"status":"error","message":"..."}`
#[wasm_bindgen]
pub fn image_to_grid(
    image_bytes: &[u8],
    grid_width: u32,
    grid_height: u32,
    smooth_strength: f32,
    threshold: u8,
    edge_strength: f32,
    noise_removal: u32,
) -> String {
    let params = nonogram_core::ImageConvertParams {
        grid_width,
        grid_height,
        smooth_strength,
        threshold,
        edge_strength,
        noise_removal,
    };

    let grid = match nonogram_core::image_to_grid(image_bytes, &params) {
        Ok(g) => g,
        Err(e) => return error_json(e.to_string()),
    };

    let grid_json_str = match grid_to_json(&grid) {
        Ok(s) => s,
        Err(e) => return error_json(e.to_string()),
    };

    let grid_value: serde_json::Value = match serde_json::from_str(&grid_json_str) {
        Ok(v) => v,
        Err(e) => return error_json(e.to_string()),
    };

    let dto = OkGridResponseDto {
        status: "ok",
        grid: grid_value,
    };
    serde_json::to_string(&dto).expect("OkGridResponseDto always serializes")
}

/// Solve a nonogram puzzle given as a JSON string.
///
/// # Parameters
/// - `puzzle_json`: JSON string with `{"row_clues": [[u32]], "col_clues": [[u32]]}` format
///
/// # Returns
/// On success: solution JSON string defined by nonogram-format
/// On failure: `{"status": "error", "message": "<error>"}` JSON string
#[wasm_bindgen]
pub fn solve(puzzle_json: &str) -> String {
    let puzzle = match puzzle_from_json(puzzle_json) {
        Ok(p) => p,
        Err(e) => return error_json(e.to_string()),
    };

    let result = ProbingSolver.solve(&puzzle);

    match result_to_json(&result) {
        Ok(json) => json,
        Err(e) => error_json(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Task 3.1: 正常系テスト ---

    #[test]
    fn solve_unique_solution_returns_valid_json() {
        // 1x1 パズル: row=[1], col=[1] → 唯一解
        let json = r#"{"row_clues":[[1]],"col_clues":[[1]]}"#;
        let result = solve(json);
        let v: serde_json::Value = serde_json::from_str(&result).expect("must be valid JSON");
        assert_eq!(v["status"], "unique");
    }

    #[test]
    fn solve_no_solution_returns_valid_json() {
        // 2行1列: row=[1],[1] → 各行1マス必要、col=[1] → 列全体で1マスしか埋められない → 解なし
        let json = r#"{"row_clues":[[1],[1]],"col_clues":[[1]]}"#;
        let result = solve(json);
        let v: serde_json::Value = serde_json::from_str(&result).expect("must be valid JSON");
        assert_eq!(v["status"], "none");
        assert_eq!(v["solutions"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn solve_multiple_solutions_returns_valid_json() {
        // 2x2 で row=[1],[1], col=[1],[1] → 複数解
        let json = r#"{"row_clues":[[1],[1]],"col_clues":[[1],[1]]}"#;
        let result = solve(json);
        let v: serde_json::Value = serde_json::from_str(&result).expect("must be valid JSON");
        let status = v["status"].as_str().unwrap();
        assert!(
            status == "unique" || status == "multiple",
            "expected unique or multiple, got {status}"
        );
    }

    #[test]
    fn solve_always_returns_valid_json_string() {
        let inputs = [
            r#"{"row_clues":[[1]],"col_clues":[[1]]}"#,
            r#"{"row_clues":[[1,2],[3]],"col_clues":[[1],[2],[1],[1]]}"#,
            "not json at all",
            r#"{"row_clues":[]}"#,
        ];
        for input in inputs {
            let result = solve(input);
            assert!(
                serde_json::from_str::<serde_json::Value>(&result).is_ok(),
                "not valid JSON for input: {input}\noutput: {result}"
            );
        }
    }

    // --- Task 3.2: エラー系テスト ---

    #[test]
    fn solve_invalid_json_returns_error_json() {
        let result = solve("not valid json");
        let v: serde_json::Value = serde_json::from_str(&result).expect("must be valid JSON");
        assert_eq!(v["status"], "error");
        assert!(v["message"].as_str().is_some());
    }

    #[test]
    fn solve_missing_field_returns_error_json() {
        let result = solve(r#"{"row_clues":[[1]]}"#);
        let v: serde_json::Value = serde_json::from_str(&result).expect("must be valid JSON");
        assert_eq!(v["status"], "error");
        assert!(v["message"].as_str().is_some());
    }

    #[test]
    fn solve_empty_clue_lists_returns_error_json() {
        // row_clues と col_clues が空配列 → Puzzle::new が EmptyClueList エラー
        let result = solve(r#"{"row_clues":[],"col_clues":[]}"#);
        let v: serde_json::Value = serde_json::from_str(&result).expect("must be valid JSON");
        assert_eq!(v["status"], "error");
        assert!(v["message"].as_str().is_some());
    }

    #[test]
    fn solve_error_response_has_status_and_message_fields() {
        let result = solve("{}");
        let v: serde_json::Value = serde_json::from_str(&result).expect("must be valid JSON");
        assert_eq!(v["status"], "error");
        assert!(
            v.get("message").is_some(),
            "error response must have 'message' field"
        );
    }

    // --- Task 4: image_to_grid WASM バインディングテスト ---

    fn make_white_png(width: u32, height: u32) -> Vec<u8> {
        use image::{DynamicImage, GrayImage, Luma};
        let img = GrayImage::from_pixel(width, height, Luma([255u8]));
        let dyn_img = DynamicImage::ImageLuma8(img);
        let mut bytes = Vec::new();
        dyn_img
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("PNG 生成失敗");
        bytes
    }

    #[test]
    fn image_to_grid_valid_png_returns_ok_status() {
        let png = make_white_png(10, 10);
        let result = image_to_grid(&png, 5, 5, 0.0, 128, 0.0, 0);
        let v: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(v["status"], "ok");
        assert!(v["grid"].is_object());
    }

    #[test]
    fn image_to_grid_returns_correct_dimensions() {
        let png = make_white_png(20, 20);
        let result = image_to_grid(&png, 10, 8, 0.0, 128, 0.0, 0);
        let v: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(v["status"], "ok");
        assert_eq!(v["grid"]["cols"], 10);
        assert_eq!(v["grid"]["rows"], 8);
    }

    #[test]
    fn image_to_grid_invalid_bytes_returns_error_status() {
        let result = image_to_grid(b"not an image", 5, 5, 0.0, 128, 0.0, 0);
        let v: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(v["status"], "error");
        assert!(v["message"].as_str().is_some());
    }

    #[test]
    fn image_to_grid_always_returns_valid_json() {
        let result = image_to_grid(b"garbage bytes", 5, 5, 0.0, 128, 0.0, 0);
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }
}
