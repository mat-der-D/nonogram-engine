# リサーチ＆設計決定ログ: nonogram-wasm

---
**Purpose**: ディスカバリフェーズで得た知見・調査結果・設計根拠を記録する。

---

## Summary

- **Feature**: `nonogram-wasm`
- **Discovery Scope**: Extension（既存スタブクレートへのwasm-bindgen統合）
- **Key Findings**:
  - `wasm-bindgen 0.2.114`（2026-02-27リリース）が最新安定版。`#[wasm_bindgen]` マクロで `&str` → `String` シグネチャをそのまま使用可能
  - `nonogram-format` はすでに `puzzle_from_json` / `result_to_json` を実装済みであり、WASMバインディング層でJSON処理を再実装する必要はない
  - `nonogram-wasm/src/lib.rs` は現状スタブ（`add` 関数のみ）。`Cargo.toml` には依存が一切ない

---

## Research Log

### wasm-bindgen バージョンと API

- **Context**: `nonogram-wasm` の `Cargo.toml` に追加する wasm-bindgen バージョンの決定
- **Sources Consulted**: [crates.io/crates/wasm-bindgen](https://crates.io/crates/wasm-bindgen)、[docs.rs/wasm-bindgen](https://docs.rs/wasm-bindgen)
- **Findings**:
  - 最新安定版: `0.2.114`（2026-02-27）
  - `#[wasm_bindgen]` を付与した `fn solve(puzzle_json: &str) -> String` がそのまま JS に公開される
  - `crate-type = ["cdylib", "rlib"]` が必須（`cdylib` でWASMバイナリ、`rlib` でRustテスト）
  - `wasm-pack build --target bundler` が ES モジュール（`.js` + `.d.ts` + `.wasm`）を生成する
- **Implications**: バージョン指定は `"0.2"` で十分（semver互換）

### nonogram-format の既存 API

- **Context**: JSON処理の委譲方法を確認
- **Sources Consulted**: `crates/nonogram-format/src/lib.rs`（コードリーディング）
- **Findings**:
  - `puzzle_from_json(json: &str) -> Result<Puzzle, FormatError>`: 入力JSONをパースして `Puzzle` を返す
  - `result_to_json(result: &SolveResult) -> Result<String, FormatError>`: `SolveResult` をJSON文字列に変換
  - `FormatError` は `Json`（serde_json）・`InvalidPuzzle`（core::Error）・`UnknownCell` の3バリアント
  - 成功時の JSON 形式: `{"status": "unique|multiple|none", "solutions": [...]}`
- **Implications**: WASMバインディング層は `FormatError` を捕捉してエラーJSONに変換するだけでよい

### CI ワークフローの現状

- **Context**: Requirement 3.5 が CI `build-wasm` ジョブを要求している
- **Sources Consulted**: `.github/workflows/` ディレクトリ（ファイルなし）
- **Findings**: 現時点で `.github/workflows/ci.yml` は存在しない
- **Implications**: CIワークフローファイルの新規作成が必要。設計スコープには含めるが、実装タスクとして別途対応する

---

## Architecture Pattern Evaluation

| オプション | 説明 | 強み | リスク・制限 | 備考 |
|------------|------|------|--------------|------|
| Thin Wrapper（採用） | `#[wasm_bindgen]` 関数が直接 nonogram-format を呼び出す | シンプル、依存最小、テスト容易 | なし | ステアリング方針「Webは完全クライアントサイド」と一致 |
| 中間アダプター層 | WasmAdapterトレイトを定義してモック化 | テスト容易性向上 | 過剰設計（1関数のみ） | スコープ外 |

---

## Design Decisions

### Decision: エラー応答のJSON形式

- **Context**: Requirement 2.3 が `status: "error"` と `message` フィールドを要求
- **Alternatives Considered**:
  1. `throw` せず空文字列を返す — エラー内容が不明になる
  2. `{"status": "error", "message": "..."}` の統一形式 — 成功時と同じトップレベル構造
- **Selected Approach**: `{"status": "error", "message": "..."}` を返す
- **Rationale**: 成功レスポンス（status フィールドを持つ）と一貫した構造。フロントエンドが単一の `status` フィールドで分岐できる
- **Trade-offs**: `FormatError::Display` をメッセージに使用するためエラー詳細が英語になる
- **Follow-up**: フロントエンドの国際化が必要な場合はエラーコードフィールドの追加を検討

### Decision: `console_error_panic_hook` の不採用

- **Context**: WASMでのパニック時のデバッグ支援
- **Alternatives Considered**:
  1. `console_error_panic_hook` を追加 — 開発時のデバッグが容易
  2. 採用しない — 依存を最小に保つ
- **Selected Approach**: 採用しない（Requirement 1.5 が nonogram-core/format のみの依存を要求）
- **Rationale**: `solve` 関数はパニックしない設計（Result で全エラーハンドリング）のため不要
- **Trade-offs**: パニックが万一発生した場合のデバッグが困難

---

## Risks & Mitigations

- **wasm-bindgen バージョンと wasm-pack CLIの不一致** — CI でインストールする wasm-pack バージョンを固定（`cargo install wasm-pack --version x.y.z`）
- **`result_to_json` が `FormatError::UnknownCell` を返す可能性** — CspSolver は全セルを解決するため実運用上は発生しないが、エラーパスで捕捉済み
- **CIワークフローが存在しない** — タスク実装時に `.github/workflows/ci.yml` を新規作成する

---

## References

- [wasm-bindgen crates.io](https://crates.io/crates/wasm-bindgen) — バージョン確認
- [wasm-bindgen docs.rs](https://docs.rs/wasm-bindgen) — API リファレンス
- [wasm-pack GitHub](https://github.com/rustwasm/wasm-pack) — ビルドツール
