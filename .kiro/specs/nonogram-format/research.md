# リサーチ & 設計決定ログ

---
**目的**: 技術設計の根拠となる調査結果・アーキテクチャ検討・設計判断を記録する。

---

## サマリー
- **機能**: `nonogram-format`
- **ディスカバリースコープ**: Extension（既存クレートの実装）
- **主要な発見**:
  - `nonogram-format` クレートは既に Cargo workspace に登録済みだが、実装は空スタブ（`pub fn add(...)` のみ）
  - `nonogram-core` は `thiserror = "2.0.18"` を使用しており、`serde` / `serde_json` は未導入
  - `nonogram-core` の公開 API（`Puzzle`, `Clue`, `Cell`, `Grid`, `SolveResult`）はすべてデザインに必要な操作を提供している

## リサーチログ

### nonogram-core の公開型分析

- **コンテキスト**: JSON ↔ コア型の変換に使用するインターフェースを確認するため
- **調査対象**: `crates/nonogram-core/src/` 配下の各ファイル
- **発見事項**:
  - `Clue::new(Vec<u32>)` — ブロック長スライスからクルーを構築。空 `vec![]` は有効（ゼロブロック行）
  - `Puzzle::new(Vec<Clue>, Vec<Clue>)` — 空リスト・クルー超過を検出しエラーを返す
  - `Grid::row(index)` — `&[Cell]` を返す。行優先アクセス可能
  - `SolveResult` — `NoSolution` / `UniqueSolution(Grid)` / `MultipleSolutions(Vec<Grid>)` の3バリアント
  - `Cell` — `Unknown` / `Filled` / `Blank` の3バリアント（`Unknown` は解答に含まれない）
- **影響**:
  - `puzzle_from_json` は `Vec<Vec<u32>>` → `Vec<Clue>` → `Puzzle::new()` の変換パスで実装できる
  - `result_to_json` はグリッド走査時に `Unknown` を検出してエラーにする必要がある

### serde/serde_json の採用

- **コンテキスト**: JSON 変換ライブラリの選定
- **調査対象**: ステアリング `tech.md`（`serde` / `serde_json` がメインライブラリとして記載）
- **発見事項**:
  - ステアリングで `serde` / `serde_json` が標準採用として明記されている
  - `serde` + `serde_json` を使えば `#[derive(Deserialize)]` / `#[derive(Serialize)]` で内部 DTO 型を簡潔に定義できる
  - `serde_json` はフィールド欠落・型不一致・数値範囲外を自動検出する（要件 1.4 / 1.5 をライブラリが処理）
- **影響**:
  - 依存追加: `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`
  - `nonogram-core` の型には Serde derive を追加しない（依存方向の維持）

### thiserror によるエラー型定義

- **コンテキスト**: `FormatError` 型の設計
- **調査対象**: `crates/nonogram-core/Cargo.toml`（`thiserror = "2.0.18"` 使用確認）
- **発見事項**:
  - `nonogram-core` と同バージョン `thiserror = "2.0.18"` を使用することで workspace 内の依存解決が安定する
  - `#[from]` による `serde_json::Error` / `nonogram_core::Error` の自動変換が `?` 演算子との相性が良い
- **影響**:
  - `FormatError` は3バリアント: `Json(serde_json::Error)`, `InvalidPuzzle(nonogram_core::Error)`, `UnknownCell`

### モジュール構造の決定

- **コンテキスト**: クレートのファイル構成をどう設計するか
- **調査対象**: ステアリング `tech.md`（`mod.rs` を使わないモダン Rust パターン）
- **発見事項**:
  - `nonogram-format` の公開 API は関数3つ（`puzzle_from_json`, `result_to_json`, `generate_template`）とエラー型1つのみ
  - コードボリュームが小さいため、`lib.rs` + `error.rs` の2ファイル構成が適切
  - 将来の拡張に備えて `lib.rs` 内に内部 DTO 型を定義する
- **影響**:
  - ファイル構成: `src/lib.rs`（公開 API + 内部 DTO）, `src/error.rs`（`FormatError`）

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク / 制限 | 備考 |
|-----------|------|------|--------------|------|
| 薄いアダプター層 | 内部 DTO ↔ コア型の変換のみ | シンプル・テスト容易・境界明確 | なし | **採用** |
| Serde derive を nonogram-core に追加 | core 型に直接 `Serialize/Deserialize` を実装 | 変換コード不要 | 依存方向違反（core がフォーマット層に依存） | **不採用** |
| カスタム From/Into トレイト | DTO と core 型の From 実装を定義 | 変換コードが型安全 | 関数ベースより複雑 | 規模に対してオーバーエンジニアリング |

## 設計決定

### Decision: 内部 DTO 型を非公開にする

- **コンテキスト**: JSON ↔ 型変換に `PuzzleDto` / `SolutionDto` が必要
- **検討した代替案**:
  1. DTO 型を `pub` にして外部から再利用可能にする
  2. DTO 型を `pub(crate)` または非公開にする
- **選択したアプローチ**: DTO 型は公開しない（`lib.rs` の内部型）
- **根拠**: 呼び出し元は常に `nonogram-core` 型または JSON 文字列を扱うため、DTO 型を公開する必要がない。公開 API をミニマルに保つことでクレートの安定性が向上する
- **トレードオフ**: DTO 型の外部再利用不可（現時点では不要）

### Decision: `generate_template` はエラーを返さない

- **コンテキスト**: テンプレート生成関数のシグネチャ設計
- **検討した代替案**:
  1. `Result<String, FormatError>` を返す
  2. `String` を直接返す（`rows`・`cols` は有効な `usize`）
- **選択したアプローチ**: `String` を直接返す
- **根拠**: 任意の `rows` / `cols` の組み合わせで有効なテンプレートを生成できる。空クルーは `Clue::new(vec![])` が有効なため、`Puzzle` 構築エラーの可能性もない
- **トレードオフ**: `rows = 0` や `cols = 0` の場合は空の `row_clues`/`col_clues` が生成されるが、それを `puzzle_from_json` に渡すと `InvalidPuzzle(Error::EmptyClueList)` が返るため問題ない（呼び出し元責務）

### Decision: `Unknown` セルのシリアライズを即時エラーにする

- **コンテキスト**: `Cell::Unknown` を含むグリッドのシリアライズ挙動
- **検討した代替案**:
  1. `Unknown` → `null` や特殊値としてシリアライズする
  2. `Unknown` が存在したら即時 `FormatError::UnknownCell` を返す
- **選択したアプローチ**: 即時エラー
- **根拠**: 要件 2.6 で明示されている。`Unknown` セルは未完成の解答を示すため、完成した解答 JSON に含めることは論理的に誤り

## リスク & 緩和策

- `generate_template(0, N)` や `generate_template(M, 0)` を `puzzle_from_json` に渡すと `EmptyClueList` エラーになる — テスト要件に明示的なテストケースは不要（設計上の仕様）
- `serde_json::to_string` が `Vec<Vec<Vec<bool>>>` に対してパニックする可能性 — 実運用上はなし（プリミティブ型のベクタは常にシリアライズ可能）

## 参照

- `crates/nonogram-core/src/` — コア型の API 確認
- `crates/nonogram-core/Cargo.toml` — thiserror バージョン確認
- `.kiro/steering/tech.md` — serde/serde_json 採用方針、モジュール構成規則
- `.kiro/steering/structure.md` — 依存方向ルール
