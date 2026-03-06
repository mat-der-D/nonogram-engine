# 要件ドキュメント

## はじめに

`nonogram-wasm` は `nonogram-core` のソルバロジックを WebAssembly 経由でブラウザから利用可能にするバインディングクレートである。`wasm-bindgen` を用いて JavaScript 向けの API をエクスポートし、`apps/web` の React フロントエンドがバックエンドなしでノノグラムパズルを解けるようにする。

**前提スペック:** `nonogram-format` スペックが承認・実装済みであること（JSON 変換ロジックはそちらで定義される）。

---

## 要件

### Requirement 1: WASM バインディング API の提供

**目標:** Web アプリケーション開発者として、JavaScript から直接呼び出せる `solve` 関数が欲しい。これにより、バックエンドなしでブラウザ上でノノグラムを解ける。

#### 受け入れ基準

1. The `nonogram-wasm` shall `#[wasm_bindgen]` を用いて `solve(puzzle_json: &str) -> String` 関数を JavaScript に公開する。
2. When `solve` が有効な問題 JSON 文字列を受け取ったとき、the `nonogram-wasm` shall `nonogram-core::CspSolver` を使ってパズルを解く。
3. When `solve` がパズルを解いたとき、the `nonogram-wasm` shall 解答 JSON 文字列を返す。
4. The `nonogram-wasm` shall JSON の解析・生成を `nonogram-format` に委譲し、直接的な JSON 処理を行わない。
5. The `nonogram-wasm` shall `nonogram-core` と `nonogram-format` のみに依存する。

---

### Requirement 2: エラーハンドリング

**目標:** フロントエンド開発者として、不正な入力に対しても安全な JSON 応答が返ってくることを期待する。これにより、アプリがパニックせずエラーを表示できる。

#### 受け入れ基準

1. If `solve` が JSON として無効な文字列を受け取ったとき、the `nonogram-wasm` shall パニックせず、エラーを示す JSON 文字列を返す。
2. If `solve` がパズルバリデーションに失敗する問題 JSON を受け取ったとき、the `nonogram-wasm` shall エラーを示す JSON 文字列を返す。
3. The `nonogram-wasm` shall エラー応答に `"status": "error"` フィールドと、エラー内容を示す `"message"` フィールドを含む。

---

### Requirement 3: ビルドとパッケージング

**目標:** Web 開発者として、`wasm-pack build` でビルドできるクレートが欲しい。これにより、`apps/web` から npm パッケージとして利用できる。

#### 受け入れ基準

1. The `nonogram-wasm` shall `wasm-pack build --target bundler crates/nonogram-wasm` でビルドが成功する。
2. The `nonogram-wasm` shall `Cargo.toml` の `[lib]` セクションに `crate-type = ["cdylib", "rlib"]` を設定する。
3. The `nonogram-wasm` shall `wasm-bindgen` を依存として `Cargo.toml` に含める。
4. When ビルドが完了したとき、the `nonogram-wasm` shall `solve` 関数が ES モジュールとして公開されている。
5. The `nonogram-wasm` shall CI ワークフロー（`.github/workflows/ci.yml`）の `build-wasm` ジョブでビルドが成功する。
