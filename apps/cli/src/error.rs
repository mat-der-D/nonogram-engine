/// CLI 全体のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Image decode error: {0}")]
    ImageDecode(String),
}
