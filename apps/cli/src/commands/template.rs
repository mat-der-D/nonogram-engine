use std::path::PathBuf;

use clap::Args;
use nonogram_format::generate_template;

use crate::error::CliError;
use crate::io::write_output;

#[derive(Args, Debug)]
pub struct TemplateArgs {
    /// 行数（1 以上）
    #[arg(long, value_name = "N")]
    pub rows: usize,
    /// 列数（1 以上）
    #[arg(long, value_name = "M")]
    pub cols: usize,
    /// 出力ファイルパス（省略時は stdout）
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

pub fn run_template(args: TemplateArgs) -> Result<(), CliError> {
    if args.rows == 0 || args.cols == 0 {
        return Err(CliError::Validation(
            "rows and cols must be 1 or greater".to_string(),
        ));
    }
    let json = generate_template(args.rows, args.cols);
    write_output(args.output.as_deref(), &json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_template_rows_zero_returns_validation_error() {
        let args = TemplateArgs {
            rows: 0,
            cols: 5,
            output: None,
        };
        let err = run_template(args).unwrap_err();
        assert!(matches!(err, CliError::Validation(_)));
    }

    #[test]
    fn run_template_cols_zero_returns_validation_error() {
        let args = TemplateArgs {
            rows: 5,
            cols: 0,
            output: None,
        };
        let err = run_template(args).unwrap_err();
        assert!(matches!(err, CliError::Validation(_)));
    }

    #[test]
    fn run_template_both_zero_returns_validation_error() {
        let args = TemplateArgs {
            rows: 0,
            cols: 0,
            output: None,
        };
        let err = run_template(args).unwrap_err();
        assert!(matches!(err, CliError::Validation(_)));
    }

    #[test]
    fn run_template_generates_correct_structure() {
        let json = generate_template(3, 4);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["row_clues"].as_array().unwrap().len(), 3);
        assert_eq!(v["col_clues"].as_array().unwrap().len(), 4);
    }
}
