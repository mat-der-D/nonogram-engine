# リサーチ・設計決定ログ

---

## サマリー

- **フィーチャー**: `nonogram-cli`
- **ディスカバリースコープ**: Extension（拡張）— `apps/cli` のスタブ実装を、既存ライブラリを接続する形で完全実装する
- **主要所見**:
  - `nonogram-core` と `nonogram-format` には必要な機能がほぼ揃っており、CLI は薄いアダプター層として実装できる
  - ドットグリッド JSON フォーマット（`[[bool,...],...]`）は `nonogram-format` の `grid_to_json`（`{"rows":N,"cols":M,"cells":...}` 形式）とは別物であり、CLI 側で独自シリアライズが必要
  - `grid-to-puzzle` のクルー計算ロジック（ランレングス符号化）は既存ライブラリに存在しないため CLI 内で実装が必要

---

## リサーチログ

### 既存ライブラリインターフェースの調査

- **背景**: CLI が依存するライブラリの公開 API を把握し、どの機能を再利用できるか確認する
- **調査対象**: `crates/nonogram-core/src/lib.rs`, `crates/nonogram-format/src/lib.rs`
- **所見**:
  - `nonogram-format` 公開関数:
    - `puzzle_from_json(json: &str) -> Result<Puzzle, FormatError>` — パズル JSON 解析
    - `result_to_json(result: &SolveResult) -> Result<String, FormatError>` — 解答 JSON 生成
    - `generate_template(rows: usize, cols: usize) -> String` — 空テンプレート生成
    - `grid_to_json(grid: &Grid) -> Result<String, FormatError>` — グリッドシリアライズ（`{"rows","cols","cells"}` 形式）
    - `json_to_grid(json: &str) -> Result<Grid, FormatError>` — グリッドデシリアライズ（同上形式）
  - `nonogram-core` 公開型・関数:
    - `Solver` トレイト: `fn solve(&self, puzzle: &Puzzle) -> SolveResult`
    - `CspSolver`, `ProbingSolver` — `Solver` を実装
    - `image_to_grid(bytes: &[u8], params: &ImageConvertParams) -> Result<Grid, ImageError>`
    - `ImageConvertParams { grid_width, grid_height, smooth_strength, threshold, edge_strength, noise_removal }`
    - `Grid::row(index) -> &[Cell]`, `Grid::col(index) -> Vec<Cell>`, `Grid::height()`, `Grid::width()`
    - `SolveResult::NoSolution`, `UniqueSolution(Grid)`, `MultipleSolutions(Vec<Grid>)`
- **含意**: CLI は既存 API を呼び出すだけで、ソルバ・画像変換・フォーマット変換の実装を重複させる必要はない

### ドットグリッド JSON フォーマットの差異

- **背景**: `convert` コマンドの出力と `grid-to-puzzle` の入力は `[[bool,...],...]` 形式だが、`grid_to_json` は `{"rows":N,"cols":M,"cells":...}` 形式を使う
- **所見**:
  - `result_to_json` の内部実装 (`SolutionDto::solutions: Vec<Vec<Vec<bool>>>`) は `[[bool,...],...]` を直接生成している
  - 要件 3.10 が指定する「SolveResult の solution グリッドと同一フォーマット」は `solutions[n]` の内側の `Vec<Vec<bool>>` 部分に相当する
  - `nonogram-format` は `Vec<Vec<bool>>` ↔ JSON の相互変換を公開していない
- **含意**: CLI 側で `serde_json` を使い `Vec<Vec<bool>>` を直接シリアライズ/デシリアライズする必要がある。`grid_to_json`/`json_to_grid` は `convert` コマンドの dot-grid フォーマットには**使用しない**

### `grid-to-puzzle` クルー計算の空白

- **背景**: 要件 4.3–4.5 ではグリッドから行・列クルーを計算する必要があるが、既存ライブラリにこの関数は存在しない
- **所見**:
  - `nonogram-core` と `nonogram-format` にはグリッド → クルー変換の公開 API がない
  - アルゴリズムは単純なランレングス符号化（連続 `true` の連長を列挙）
- **含意**: CLI の `grid_to_puzzle` モジュール内で `compute_clue(line: &[bool]) -> Vec<u32>` 関数を実装する

### `apps/cli` 現状の調査

- **所見**:
  - `apps/cli/src/main.rs`: `println!("Hello, world!")` のみ
  - `apps/cli/Cargo.toml`: 依存関係が空
  - ワークスペース `Cargo.toml` に `apps/cli` はすでに登録済み
- **含意**: main.rs は完全に書き直す。Cargo.toml に `clap`, `serde_json`, `nonogram-core`, `nonogram-format` を追加する

### `clap` バージョン確認

- **所見**: ステアリング (`tech.md`) に `clap` が CLI 引数パースの標準として明記。最新安定版は v4.x（derive feature 利用可能）
- **含意**: `clap = { version = "4", features = ["derive"] }` を使用する。`#[derive(Parser, Subcommand, Args)]` による宣言的 CLI 定義

---

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク / 制限 | メモ |
|-----------|------|------|--------------|------|
| 薄いアダプター（採用） | CLI は clap で引数を受け取り、ライブラリ呼び出しに変換するだけ | コードが少ない、ライブラリ側の機能を最大活用 | ライブラリ API 変更の影響を受ける | 既存ステアリングの「アプリ層はクレートを組み合わせる」方針に合致 |
| 厚い CLI 層 | CLI がロジックを持ち、ライブラリは型のみ使用 | 独立性が高い | 機能重複、メンテコスト増大 | 不採用 |
| サービス層の追加 | CLI ↔ サービス ↔ ライブラリ の 3 層 | テスタビリティ向上 | 過剰な抽象化 | 不採用（YAGNI） |

---

## 設計決定

### 決定: モジュール構成（mod.rs 不使用）

- **背景**: ステアリング (`tech.md`) が `mod.rs` 方式を非推奨とし、`モジュール名.rs` + 同名ディレクトリを推奨している
- **検討案**:
  1. `commands/mod.rs` — 非推奨、不採用
  2. `commands.rs` + `commands/solve.rs` 等 — ステアリング準拠
- **採用方針**: `commands.rs`（モジュール宣言ファイル）+ `commands/` サブディレクトリ構成を採用
- **理由**: ステアリングの規約に完全準拠し、一貫性を確保
- **トレードオフ**: 宣言ファイルが増えるが、Rust 2018 以降の標準的スタイル

### 決定: ドットグリッドの直接シリアライズ

- **背景**: `convert` 出力と `grid-to-puzzle` 入力は `[[bool,...],...]` 形式だが、`nonogram-format` の `grid_to_json`/`json_to_grid` はこれと異なる形式を使用する
- **検討案**:
  1. `nonogram-format` に新関数追加 — ライブラリ変更が必要、スコープ外
  2. CLI 側で `serde_json` を使い `Vec<Vec<bool>>` を直接扱う — シンプル、独立
- **採用方針**: CLI の `io.rs` または各コマンドモジュール内で `serde_json::to_string(&dot_grid)` / `serde_json::from_str::<Vec<Vec<bool>>>(&json)` を直接使用
- **理由**: ライブラリ変更なしで要件を満たせる。フォーマット変換は CLI の責務として適切

### 決定: エラーハンドリング戦略

- **背景**: CLI はすべてのエラーを stderr に出力し、ゼロでない終了コードで終了する必要がある
- **検討案**:
  1. `anyhow` クレート使用 — 簡単だが型情報が失われる
  2. `thiserror` で CliError 型定義 — 型安全、テスト容易
  3. `Box<dyn Error>` — 簡単だが構造化エラーなし
- **採用方針**: `CliError` enum を定義し、`main` で `process::exit(1)` を呼ぶ。`thiserror` を使用
- **理由**: エラーカテゴリ（IO / パース / バリデーション）を区別でき、テスト可能

---

## リスクと軽減策

- **入力バリデーション漏れ** — `--rows`/`--cols` 範囲外、`--grid-width`/`--grid-height` 範囲外。軽減: clap の `value_parser` でバリデーション、またはコマンドハンドラで明示的チェック
- **stdin と `--input` の競合** — ユーザーが両方指定した場合。軽減: `--input` を優先し、stdin は `--input` 省略時のみ使用
- **空行・空列の画像変換結果** — すべて blank のグリッドで `grid-to-puzzle` を実行すると空クルーリストになる。軽減: 要件 4.5 に従い空配列 `[]` を正常出力とする
- **大きな画像のメモリ使用** — `image_to_grid` は全バイトをメモリに読み込む。軽減: 現時点では許容（パフォーマンス要件なし）

---

## 参照

- `crates/nonogram-core/src/lib.rs` — コア型・ソルバ公開 API
- `crates/nonogram-format/src/lib.rs` — JSON 変換 API 全文
- `crates/nonogram-core/src/image/convert.rs` — `ImageConvertParams` 定義
- `crates/nonogram-core/src/grid.rs` — `Grid::row()`, `Grid::col()` API
- `.kiro/steering/tech.md` — Rust モジュール規約、clap 採用方針
- `.kiro/steering/structure.md` — `apps/cli` の位置づけ
