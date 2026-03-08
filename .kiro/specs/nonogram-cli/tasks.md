# 実装計画

- [x] 1. プロジェクト基盤をセットアップする
- [x] 1.1 Cargo.toml に依存関係を追加する
  - `apps/cli/Cargo.toml` に `clap`（derive feature）、`serde_json`、`thiserror`、`nonogram-core`、`nonogram-format` を追加する
  - ワークスペースの既存バージョン制約（`thiserror` v2）に合わせる
  - _Requirements: 5.7_

- [x] 1.2 (P) CliError 型を実装する
  - `Io`（`std::io::Error` から変換）、`Parse`、`Validation`、`ImageDecode` の 4 バリアントを定義する
  - `thiserror` の `#[derive(Error)]` を使い、人間が読みやすいエラーメッセージを付与する
  - _Requirements: 5.5, 5.6_

- [x] 1.3 (P) I/O ユーティリティを実装する
  - `read_input(path: Option<&Path>)`: パスが `None` のとき stdin から文字列を読み込む
  - `read_bytes(path: &Path)`: ファイルからバイト列を読み込む（画像用）
  - `write_output(path: Option<&Path>, content: &str)`: パスが `None` のとき stdout に書き込む
  - I/O エラーは `CliError::Io` に変換する
  - _Requirements: 5.6, 5.8_

- [x] 2. (P) CLI エントリポイントを実装する
  - `main.rs` で `Cli::parse()` を呼び、`Commands` enum をマッチして各ハンドラを呼び出す
  - `commands.rs` で `pub mod solve; pub mod template; pub mod convert; pub mod grid_to_puzzle;` を宣言する
  - `CliError` 発生時は stderr にメッセージを表示し `process::exit(1)` を呼ぶ
  - clap の derive feature（`#[derive(Parser)]`、`#[derive(Subcommand)]`）で `Cli` 構造体と `Commands` enum を定義する
  - ツール名を `nonokit`、サブコマンドは `solve`、`template`、`convert`、`grid-to-puzzle` とする
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7_

- [x] 3. (P) solve コマンドを実装する
- [x] 3.1 引数定義と入出力ハンドリングを実装する
  - `SolveArgs` 構造体に `--input`（省略可）、`--output`（省略可）、`--solver`（`csp` | `probing`、デフォルト `csp`）を定義する
  - `run_solve()` で `IoUtils::read_input` を使い、`--input` の有無に応じてファイルまたは stdin から JSON を読み込む
  - 読み込んだ JSON を `nonogram-format::puzzle_from_json` でパースし、`FormatError` を `CliError::Parse` に変換する
  - 解答 JSON を `nonogram-format::result_to_json` でシリアライズし、`IoUtils::write_output` で出力する
  - _Requirements: 1.1, 1.2, 1.3, 1.10, 1.11, 1.12_

- [x] 3.2 ソルバ選択と解答ステータス処理を実装する
  - `SolverKind::Csp` のとき `CspSolver`、`SolverKind::Probing` のとき `ProbingSolver` を選択する
  - ソルバの `solve()` 結果から `status: "none" | "unique" | "multiple"` を判定して JSON に含める
  - 解なし・唯一解・複数解の各ケースが仕様通りの JSON フォーマットで出力されることを確認する
  - _Requirements: 1.4, 1.5, 1.6, 1.7, 1.8, 1.9_

- [x] 4. (P) template コマンドを実装する
  - `TemplateArgs` 構造体に `--rows`（必須）、`--cols`（必須）、`--output`（省略可）を定義する
  - `rows == 0` または `cols == 0` のとき `CliError::Validation` を返す
  - `nonogram-format::generate_template(rows, cols)` を呼び、結果を `IoUtils::write_output` で出力する
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 5. (P) convert コマンドを実装する
- [x] 5.1 引数定義とパラメータ検証を実装する
  - `ConvertArgs` 構造体に `--input`（必須）、`--output`（省略可）、`--smooth-strength`（デフォルト 1.0）、`--edge-strength`（デフォルト 0.3）、`--grid-width`（デフォルト 20）、`--grid-height`（デフォルト 20）、`--threshold`（デフォルト 128）、`--noise-removal`（デフォルト 0）を定義する
  - `grid_width` / `grid_height` が 5–50 の範囲外のとき `CliError::Validation` を返す
  - `smooth_strength`（0–5）、`edge_strength`（0–1）、`noise_removal`（0–20）の範囲を検証する
  - _Requirements: 3.1, 3.4, 3.5, 3.6, 3.7, 3.8, 3.12_

- [x] 5.2 画像変換と JSON 出力を実装する
  - `IoUtils::read_bytes` でバイト列を読み込み、`nonogram-core::image_to_grid(bytes, params)` を呼ぶ
  - `ImageError` を `CliError::ImageDecode` に変換する
  - `Grid` を行優先の 2D boolean 配列（`Vec<Vec<bool>>`）に変換し、`serde_json::to_string` でシリアライズする
  - アルファコンポジット・ガウスぼかし・Canny エッジ・ダウンサンプリング・閾値処理・ノイズ除去は `nonogram-core` 側に委譲する
  - 変換結果を `IoUtils::write_output` で出力する
  - _Requirements: 3.2, 3.3, 3.9, 3.10, 3.11_

- [x] 6. (P) grid-to-puzzle コマンドを実装する
- [x] 6.1 引数定義と入力パース・バリデーションを実装する
  - `GridToPuzzleArgs` 構造体に `--input`（省略可）、`--output`（省略可）を定義する
  - `run_grid_to_puzzle()` で `IoUtils::read_input` を使い stdin またはファイルから JSON を読む
  - `serde_json::from_str::<Vec<Vec<bool>>>` でドットグリッドをパースし、失敗時は `CliError::Parse` を返す
  - 行が空または行ごとの長さが一致しない場合は `CliError::Parse` を返す
  - _Requirements: 4.1, 4.2, 4.7, 4.8_

- [x] 6.2 クルー計算と JSON 出力を実装する
  - `compute_clue(line: &[bool]) -> Vec<u32>` を実装する（filled セルの連続ブロックサイズを列挙）
  - すべて blank の行・列は空配列 `[]` を返す
  - 各行・各列（転置）に `compute_clue` を適用して `row_clues`・`col_clues` を構築する
  - `{"row_clues":[...],"col_clues":[...]}` 形式で JSON をシリアライズし `IoUtils::write_output` で出力する
  - _Requirements: 4.3, 4.4, 4.5, 4.6_

- [x] 7. 単体テストを実装する
- [x] 7.1 (P) grid-to-puzzle の計算・パースロジックをテストする
  - `compute_clue`: 空行、単一ブロック、複数ブロック、全 blank のケースを検証する
  - ドットグリッドパース: 有効 JSON、不正 JSON（型不一致）、行長不一致の各ケースを検証する
  - _Requirements: 4.3, 4.4, 4.5, 4.7, 4.8_

- [x] 7.2 (P) バリデーションロジックをテストする
  - `run_template`: `rows=0` / `cols=0` が `CliError::Validation` を返すことを確認する
  - `run_convert`: `grid_width` / `grid_height` が範囲外のとき `CliError::Validation` を返すことを確認する
  - _Requirements: 2.4, 3.12_

- [x] 8. 統合テストを実装する
- [x] 8.1 solve コマンドの統合テストを実装する
  - 有効なパズル JSON（解なし・唯一解・複数解）を入力して解答 JSON の `status` と `solutions` を検証する
  - 不正 JSON を入力したとき非ゼロ終了コードで終了することを確認する
  - `--solver probing` オプションで `ProbingSolver` が使われることを確認する
  - _Requirements: 1.3, 1.7, 1.8, 1.9, 1.11_

- [x] 8.2 (P) template・grid-to-puzzle コマンドの統合テストを実装する
  - `template --rows 3 --cols 4` の出力が `row_clues` 長 3・`col_clues` 長 4 を持つことを確認する
  - `grid-to-puzzle` に convert 出力フォーマットを入力してクルーが正しく計算されることを確認する
  - `--output <path>` 指定でファイルに書き込まれ、stdout には出力されないことを確認する
  - _Requirements: 2.1, 2.2, 2.3, 4.1, 4.3, 4.4, 4.5, 4.6_

- [x] 8.3 パイプラインエンドツーエンドテストを実装する
  - `nonokit convert | nonokit grid-to-puzzle | nonokit solve` の全体フローが正常動作することを確認する
  - stdin からの入力受け付けが `solve` と `grid-to-puzzle` の両方で機能することを確認する
  - _Requirements: 5.8_
