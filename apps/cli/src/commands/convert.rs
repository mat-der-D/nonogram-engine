use std::path::PathBuf;

use clap::Args;
use nonogram_core::{Cell, ImageConvertParams, image_to_grid};

use crate::error::CliError;
use crate::io::{read_bytes, write_output};

#[derive(Args, Debug)]
pub struct ConvertArgs {
    /// 入力画像ファイルパス（必須）
    #[arg(long, value_name = "PATH")]
    pub input: PathBuf,
    /// 出力ファイルパス（省略時は stdout）
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// ガウシアンブラー強度（0–5）
    #[arg(long, default_value = "1.0")]
    pub smooth_strength: f32,
    /// エッジマージ強度（0–1）
    #[arg(long, default_value = "0.3")]
    pub edge_strength: f32,
    /// 出力グリッド幅（5–50）
    #[arg(long, default_value = "20")]
    pub grid_width: u32,
    /// 出力グリッド高さ（5–50）
    #[arg(long, default_value = "20")]
    pub grid_height: u32,
    /// 二値化閾値（0–255）
    #[arg(long, default_value = "128")]
    pub threshold: u8,
    /// ノイズ除去最小サイズ（0–20）
    #[arg(long, default_value = "0")]
    pub noise_removal: u32,
}

pub fn run_convert(args: ConvertArgs) -> Result<(), CliError> {
    // バリデーション
    if args.grid_width < 5 || args.grid_width > 50 {
        return Err(CliError::Validation(format!(
            "grid_width must be between 5 and 50, got {}",
            args.grid_width
        )));
    }
    if args.grid_height < 5 || args.grid_height > 50 {
        return Err(CliError::Validation(format!(
            "grid_height must be between 5 and 50, got {}",
            args.grid_height
        )));
    }
    if args.smooth_strength < 0.0 || args.smooth_strength > 5.0 {
        return Err(CliError::Validation(format!(
            "smooth_strength must be between 0 and 5, got {}",
            args.smooth_strength
        )));
    }
    if args.edge_strength < 0.0 || args.edge_strength > 1.0 {
        return Err(CliError::Validation(format!(
            "edge_strength must be between 0 and 1, got {}",
            args.edge_strength
        )));
    }
    if args.noise_removal > 20 {
        return Err(CliError::Validation(format!(
            "noise_removal must be between 0 and 20, got {}",
            args.noise_removal
        )));
    }

    let bytes = read_bytes(&args.input)?;
    let params = ImageConvertParams {
        grid_width: args.grid_width,
        grid_height: args.grid_height,
        smooth_strength: args.smooth_strength,
        threshold: args.threshold,
        edge_strength: args.edge_strength,
        noise_removal: args.noise_removal,
    };

    let grid = image_to_grid(&bytes, &params).map_err(|e| CliError::ImageDecode(e.to_string()))?;

    // Grid を行優先 2D boolean 配列に変換
    let dot_grid: Vec<Vec<bool>> = (0..grid.height())
        .map(|r| {
            (0..grid.width())
                .map(|c| grid.get(r, c) == Cell::Filled)
                .collect()
        })
        .collect();

    let json = serde_json::to_string(&dot_grid).map_err(|e| CliError::Parse(e.to_string()))?;

    write_output(args.output.as_deref(), &json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dummy_path() -> PathBuf {
        PathBuf::from("/nonexistent/image.png")
    }

    #[test]
    fn run_convert_grid_width_too_small_returns_validation_error() {
        let args = ConvertArgs {
            input: dummy_path(),
            output: None,
            smooth_strength: 1.0,
            edge_strength: 0.3,
            grid_width: 4,
            grid_height: 20,
            threshold: 128,
            noise_removal: 0,
        };
        let err = run_convert(args).unwrap_err();
        assert!(matches!(err, CliError::Validation(_)));
    }

    #[test]
    fn run_convert_grid_width_too_large_returns_validation_error() {
        let args = ConvertArgs {
            input: dummy_path(),
            output: None,
            smooth_strength: 1.0,
            edge_strength: 0.3,
            grid_width: 51,
            grid_height: 20,
            threshold: 128,
            noise_removal: 0,
        };
        let err = run_convert(args).unwrap_err();
        assert!(matches!(err, CliError::Validation(_)));
    }

    #[test]
    fn run_convert_grid_height_too_small_returns_validation_error() {
        let args = ConvertArgs {
            input: dummy_path(),
            output: None,
            smooth_strength: 1.0,
            edge_strength: 0.3,
            grid_width: 20,
            grid_height: 4,
            threshold: 128,
            noise_removal: 0,
        };
        let err = run_convert(args).unwrap_err();
        assert!(matches!(err, CliError::Validation(_)));
    }

    #[test]
    fn run_convert_grid_height_too_large_returns_validation_error() {
        let args = ConvertArgs {
            input: dummy_path(),
            output: None,
            smooth_strength: 1.0,
            edge_strength: 0.3,
            grid_width: 20,
            grid_height: 51,
            threshold: 128,
            noise_removal: 0,
        };
        let err = run_convert(args).unwrap_err();
        assert!(matches!(err, CliError::Validation(_)));
    }

    #[test]
    fn run_convert_noise_removal_too_large_returns_validation_error() {
        let args = ConvertArgs {
            input: dummy_path(),
            output: None,
            smooth_strength: 1.0,
            edge_strength: 0.3,
            grid_width: 20,
            grid_height: 20,
            threshold: 128,
            noise_removal: 21,
        };
        let err = run_convert(args).unwrap_err();
        assert!(matches!(err, CliError::Validation(_)));
    }
}
