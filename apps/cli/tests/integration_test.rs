use std::process::Command;

/// cargo build でバイナリパスを取得するヘルパー
fn nonokit_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe()
        .expect("failed to get test binary path")
        .parent()
        .expect("test binary has no parent")
        .to_path_buf();
    // deps/ ディレクトリ内にある場合は一つ上に上がる
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("nonokit")
}

// --- solve コマンドの統合テスト ---

#[test]
fn solve_unique_solution() {
    let json = r#"{"row_clues":[[1]],"col_clues":[[1]]}"#;
    let output = Command::new(nonokit_bin())
        .args(["solve"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn nonokit")
        .wait_with_output_and_stdin(json.as_bytes())
        .expect("failed to wait");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    assert_eq!(v["status"], "unique");
    assert_eq!(v["solutions"].as_array().unwrap().len(), 1);
}

#[test]
fn solve_no_solution() {
    // row=[1], col=[] → 解なし
    let json = r#"{"row_clues":[[1]],"col_clues":[[]]}"#;
    let output = Command::new(nonokit_bin())
        .args(["solve"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn nonokit")
        .wait_with_output_and_stdin(json.as_bytes())
        .expect("failed to wait");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    assert_eq!(v["status"], "none");
}

#[test]
fn solve_invalid_json_nonzero_exit() {
    let output = Command::new(nonokit_bin())
        .args(["solve"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn nonokit")
        .wait_with_output_and_stdin(b"not json")
        .expect("failed to wait");

    assert!(!output.status.success(), "should exit non-zero on invalid JSON");
}

#[test]
fn solve_probing_solver() {
    let json = r#"{"row_clues":[[1]],"col_clues":[[1]]}"#;
    let output = Command::new(nonokit_bin())
        .args(["solve", "--solver", "probing"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn nonokit")
        .wait_with_output_and_stdin(json.as_bytes())
        .expect("failed to wait");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    assert_eq!(v["status"], "unique");
}

// --- template コマンドの統合テスト ---

#[test]
fn template_3x4_output() {
    let output = Command::new(nonokit_bin())
        .args(["template", "--rows", "3", "--cols", "4"])
        .output()
        .expect("failed to run nonokit template");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    assert_eq!(v["row_clues"].as_array().unwrap().len(), 3);
    assert_eq!(v["col_clues"].as_array().unwrap().len(), 4);
}

#[test]
fn template_output_to_file() {
    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let path = tmp.path().to_str().unwrap().to_string();

    let output = Command::new(nonokit_bin())
        .args(["template", "--rows", "2", "--cols", "3", "--output", &path])
        .output()
        .expect("failed to run nonokit template");

    assert!(output.status.success());
    // stdout は空
    assert!(output.stdout.is_empty(), "stdout should be empty when --output is specified");
    // ファイルに JSON が書き込まれている
    let content = std::fs::read_to_string(&path).expect("failed to read output file");
    let v: serde_json::Value = serde_json::from_str(&content).expect("invalid JSON in file");
    assert_eq!(v["row_clues"].as_array().unwrap().len(), 2);
    assert_eq!(v["col_clues"].as_array().unwrap().len(), 3);
}

// --- grid-to-puzzle コマンドの統合テスト ---

#[test]
fn grid_to_puzzle_computes_clues() {
    // [[true,false],[false,true]] → row_clues=[[1],[1]], col_clues=[[1],[1]]
    let json = r#"[[true,false],[false,true]]"#;
    let output = Command::new(nonokit_bin())
        .args(["grid-to-puzzle"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn nonokit")
        .wait_with_output_and_stdin(json.as_bytes())
        .expect("failed to wait");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    let row_clues = v["row_clues"].as_array().unwrap();
    let col_clues = v["col_clues"].as_array().unwrap();
    assert_eq!(row_clues.len(), 2);
    assert_eq!(col_clues.len(), 2);
    assert_eq!(row_clues[0], serde_json::json!([1]));
    assert_eq!(row_clues[1], serde_json::json!([1]));
    assert_eq!(col_clues[0], serde_json::json!([1]));
    assert_eq!(col_clues[1], serde_json::json!([1]));
}

#[test]
fn grid_to_puzzle_output_to_file() {
    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let path = tmp.path().to_str().unwrap().to_string();
    let json = r#"[[true,false],[false,true]]"#;

    let output = Command::new(nonokit_bin())
        .args(["grid-to-puzzle", "--output", &path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn nonokit")
        .wait_with_output_and_stdin(json.as_bytes())
        .expect("failed to wait");

    assert!(output.status.success());
    assert!(output.stdout.is_empty(), "stdout should be empty when --output is specified");
    let content = std::fs::read_to_string(&path).expect("failed to read output file");
    let v: serde_json::Value = serde_json::from_str(&content).expect("invalid JSON in file");
    assert!(v["row_clues"].is_array());
    assert!(v["col_clues"].is_array());
}

// --- パイプライン E2E テスト (8.3) ---

#[test]
fn pipeline_grid_to_puzzle_then_solve() {
    // grid-to-puzzle の出力を solve に渡す
    let dot_grid = r#"[[true,false],[false,true]]"#;

    // Step 1: grid-to-puzzle
    let gtp_output = Command::new(nonokit_bin())
        .args(["grid-to-puzzle"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn grid-to-puzzle")
        .wait_with_output_and_stdin(dot_grid.as_bytes())
        .expect("failed to wait");

    assert!(gtp_output.status.success());

    // Step 2: solve
    let solve_output = Command::new(nonokit_bin())
        .args(["solve"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn solve")
        .wait_with_output_and_stdin(&gtp_output.stdout)
        .expect("failed to wait");

    assert!(solve_output.status.success(), "stderr: {}", String::from_utf8_lossy(&solve_output.stderr));
    let stdout = String::from_utf8_lossy(&solve_output.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    assert!(v["status"].is_string());
    assert!(v["solutions"].is_array());
}

// ヘルパー trait: stdin を送ってから出力を待つ
trait WaitWithStdin {
    fn wait_with_output_and_stdin(self, input: &[u8]) -> std::io::Result<std::process::Output>;
}

impl WaitWithStdin for std::process::Child {
    fn wait_with_output_and_stdin(mut self, input: &[u8]) -> std::io::Result<std::process::Output> {
        use std::io::Write;
        if let Some(mut stdin) = self.stdin.take() {
            stdin.write_all(input)?;
        }
        self.wait_with_output()
    }
}
