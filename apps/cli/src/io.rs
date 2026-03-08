use std::io::{self, Read, Write};
use std::path::Path;

use crate::error::CliError;

/// テキスト入力を読み込む。`path` が `None` のときは stdin から読む。
pub fn read_input(path: Option<&Path>) -> Result<String, CliError> {
    match path {
        Some(p) => std::fs::read_to_string(p).map_err(CliError::Io),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).map_err(CliError::Io)?;
            Ok(buf)
        }
    }
}

/// バイナリファイルを読み込む（画像用）。
pub fn read_bytes(path: &Path) -> Result<Vec<u8>, CliError> {
    std::fs::read(path).map_err(CliError::Io)
}

/// テキスト出力を書き込む。`path` が `None` のときは stdout に書く。
pub fn write_output(path: Option<&Path>, content: &str) -> Result<(), CliError> {
    match path {
        Some(p) => std::fs::write(p, content).map_err(CliError::Io),
        None => {
            io::stdout()
                .write_all(content.as_bytes())
                .map_err(CliError::Io)?;
            Ok(())
        }
    }
}
