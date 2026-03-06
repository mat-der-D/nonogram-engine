# 実装計画

- [ ] 1. Cargo.toml にビルド設定と依存関係を追加する
  - `[lib]` セクションに `crate-type = ["cdylib", "rlib"]` を設定する
  - `wasm-bindgen`・`nonogram-core`・`nonogram-format` を依存として追加する
  - _Requirements: 1.5, 3.1, 3.2, 3.3_

- [ ] 2. WASM バインディング層を実装する

- [ ] 2.1 エラー応答のデータ構造を定義する
  - `"status"` と `"message"` フィールドを持つエラー応答型を定義する
  - serde の `Serialize` を実装してシリアライズ可能にする
  - _Requirements: 2.3_

- [ ] 2.2 `solve` 関数を実装する
  - `#[wasm_bindgen]` アトリビュートで JavaScript に公開する
  - `nonogram-format` の変換関数で入力 JSON をパースし、パース失敗時はエラー応答 JSON を返す
  - `CspSolver` でパズルを解き、`nonogram-format` の変換関数で結果を JSON 化する
  - JSON 変換を `nonogram-format` に委譲し、直接 `serde_json` を使用しない
  - いずれかのステップで失敗した場合はパニックせずエラー応答 JSON を返す
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2_

- [ ] 3. ユニットテストを実装する

- [ ] 3.1 正常系テストを実装する
  - 有効な puzzle JSON で `solve` を呼び出すと解答ステータス（`unique`/`multiple`/`none`）が返ることを検証する
  - 返り値が常に有効な JSON 文字列であることを `serde_json::from_str` で確認する
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 3.2 エラー系テストを実装する
  - 不正な JSON 文字列を渡したときエラー JSON が返り、パニックしないことを検証する
  - バリデーション失敗の puzzle JSON を渡したときエラー JSON が返ることを検証する
  - エラー応答に `"status": "error"` と `"message"` フィールドが含まれることを確認する
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 4. (P) CI ワークフローに `build-wasm` ジョブを追加する
  - `.github/workflows/ci.yml` に `wasm32-unknown-unknown` ターゲットと `wasm-pack` のセットアップを含む `build-wasm` ジョブを追加する
  - `wasm-pack build --target bundler crates/nonogram-wasm` が成功することを CI 上で確認する
  - 生成された ESモジュールに `solve` 関数が含まれることを確認する
  - タスク 1 完了後、タスク 2・3 と並行して作業可能（別ファイルを編集するため）
  - _Requirements: 3.4, 3.5_
