# 調査・設計決定記録

---
**目的**: 技術設計を根拠付ける発見事項・アーキテクチャ調査・意思決定の根拠を記録する。
---

## サマリー

- **フィーチャー**: `nonogram-core`
- **ディスカバリー範囲**: 新規フィーチャー（グリーンフィールド）
- **主要な発見**:
  - DP（動的計画法）ベースの行ソルバは O(k × L) の計算量で効率的に制約伝播を実現できる。フォワード/バックワード2パスで全有効配置の交差集合を導出
  - Rust 2024 エディションでは `mod.rs` を使わない現代的なモジュール構成が推奨される。`solver.rs` + `solver/` ディレクトリの構成を採用
  - 外部依存ゼロを維持する。`Error` enum は手動で `Display` + `std::error::Error` を実装

---

## 調査ログ

### 制約伝播アルゴリズム（行ソルバ）

- **背景**: 要件 3 の `LinePropagator` の中核アルゴリズム選定
- **参照資料**:
  - [Nonogram solver - Rosetta Code](https://rosettacode.org/wiki/Nonogram_solver) — 各言語での実装例
  - [Nonograms: Combinatorial questions and algorithms](https://www.sciencedirect.com/science/article/pii/S0166218X14000080) — 学術的な計算量解析
  - [How To Solve Japanese Crosswords with Python, Rust, And WebAssembly](https://www.smartspate.com/how-to-solve-japanese-crosswords-with-python-rust-and-webassembly/) — Rust + WASM 実装事例
- **発見事項**:
  - 行ソルバの標準手法は「与えられた行/列のすべての有効配置の交差集合を計算する」方法
  - DP により各ブロックの配置可能範囲を O(k × L) で計算可能（k=ブロック数, L=行長）
  - フォワードパス（左→右）で「各ブロックをここまでに配置できるか」を計算し、バックワードパス（右→左）で同様に計算。両パスの結果から各セルが全配置で共通する値を導出
  - 行/列を順番に処理し、変化がなくなるまで反復（フィックスポイント到達）
  - Rust + WASM での実装例では、PyPy 比で1〜3桁高速
- **設計への影響**: `LinePropagator` は2パス DP を中核メソッドとし、フィックスポイント反復で全行/列を処理

### バックトラッキングと MRV ヒューリスティック

- **背景**: 要件 4, 4a のバックトラッキング設計
- **参照資料**:
  - CSP における MRV ヒューリスティック
  - [GitHub - attilaszia/nonogram](https://github.com/attilaszia/nonogram) — C++ CSP 実装
- **発見事項**:
  - ノノグラムのセルドメインは常に {Filled, Blank} の2値で、MRV を素朴に適用すると全 Unknown セルが等価
  - 実用的な MRV: 「そのセルを含む行と列の Unknown セル数の合計が最小」のセルを選択 → 最も制約の強い箇所を優先
  - 状態スナップショットは `Grid::clone()` で実現し、矛盾時にスナップショットを復元
  - DFS で1件目の解を発見後も探索を継続し、2件目発見時に `MultipleSolutions` を返して打ち切り
- **設計への影響**: `Backtracker` は Grid の clone でスナップショット管理。MRV は行/列の Unknown セル数に基づく

### Rust トレイトオブジェクト安全性

- **背景**: 要件 2.4「`dyn Solver` での交換可能利用」
- **発見事項**:
  - `dyn Solver` として使うにはトレイトメソッドが `Sized` を要求しないこと、`Self` 型を返さないことが必要
  - `solve(&self, puzzle: &Puzzle) -> SolveResult` はオブジェクト安全
  - `Clone` はオブジェクト安全でないため、`Solver` のスーパートレイトにしない
- **設計への影響**: `Solver` トレイトは `solve` メソッド1つのみ

### Rust エラー型設計

- **背景**: 要件 7 のエラーハンドリング方針
- **参照資料**:
  - [Rust Error Handling Guide 2025](https://markaicode.com/rust-error-handling-2025-guide/)
  - [How to Design Error Types with thiserror and anyhow](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view)
- **発見事項**:
  - ライブラリでは `thiserror` で明確なエラー型を提供するのが一般的
  - `nonogram-core` のエラーバリアントは2種（DimensionMismatch, ClueExceedsLength）で、手動実装のコストが低い
  - 外部依存ゼロを維持することでコンパイル時間・バイナリサイズ・サプライチェーンリスクを削減
- **設計への影響**: `Error` enum を手動実装。`thiserror` は不使用

### プロービングアルゴリズム

- **背景**: 要件 5 の `ProbingSolver` 設計
- **参照資料**: LalaFrogKK アルゴリズム（Wu et al.）、`docs/repository-plan.md` セクション 3.3
- **発見事項**:
  - プロービングは各 Unknown セルに Filled/Blank を仮定し、制約伝播を実行
  - 両仮定で同一結果となるセルは確定として確約
  - 進展がなくなるまで反復し、その後バックトラッキングに移行
  - メモリ使用量はグリッドクローン2回分（各仮定用）が追加で必要
- **設計への影響**: `ProbingSolver` は LP → Probing ループ → Backtracker の3フェーズ構造

---

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク・制限 | 備考 |
|---|---|---|---|---|
| フラットモジュール | 全ファイルを `src/` 直下に配置 | シンプル、ナビゲーション容易 | ソルバ増加時に煩雑化 | 小規模には十分 |
| 完全階層化 | `types/`, `solver/`, `internal/` に分割 | 責務分離が明確 | 初期構造が複雑 | 過剰設計の可能性 |
| **ハイブリッド（採用）** | データ型はフラット、ソルバは `solver/` サブモジュール | バランスが良い | — | steering 構造規約に適合、拡張性確保 |

---

## 設計決定

### 決定: モジュール構成

- **背景**: `nonogram-core` の内部モジュール構成
- **検討した選択肢**:
  1. 全ファイルを `src/` 直下に配置
  2. `src/types/`, `src/solver/`, `src/internal/` の3階層
  3. ハイブリッド — データ型はフラット、ソルバはサブモジュール
- **採用**: ハイブリッド構成
- **根拠**: データ型は少数で独立しているためフラットで十分。ソルバは今後増える可能性があるためサブモジュール化。steering の `mod.rs` 不使用ルールに従い `solver.rs` + `solver/` を使用
- **トレードオフ**: 若干の構造的複雑さと引き換えに拡張性を確保

### 決定: Grid の内部表現

- **背景**: `Grid` のデータ構造選定
- **検討した選択肢**:
  1. `Vec<Vec<Cell>>` — 行単位のアクセスが自然
  2. `Vec<Cell>` フラット配列 — キャッシュ効率が高い
- **採用**: `Vec<Vec<Cell>>`
- **根拠**: 行単位の制約伝播で行スライスを直接渡せる。列アクセスには `col()` メソッドで `Vec<Cell>` を生成。25×25 以下のパズルではキャッシュ効率の差は無視できる。コードの明瞭さを優先
- **トレードオフ**: 列アクセスにアロケーションが発生するが、対象パズルサイズでは問題にならない

### 決定: 外部依存ゼロ

- **背景**: `Cargo.toml` に外部依存を追加するか
- **検討した選択肢**:
  1. `thiserror` でエラー型を簡略化
  2. 外部依存なしで手動実装
- **採用**: 外部依存なし
- **根拠**: エラーバリアントが少数で手動実装コストが低い。ゼロ依存はコンパイル時間・バイナリサイズ・サプライチェーンリスクで有利
- **トレードオフ**: `Display` の手動実装が必要だがコード量は最小限

### 決定: ProbingSolver の実装方針

- **背景**: 要件 5 は「オプション」とされている
- **検討した選択肢**:
  1. Cargo フィーチャーフラグで条件コンパイル
  2. 常時コンパイル対象として実装
- **採用**: 常時コンパイル対象
- **根拠**: フィーチャーフラグは複雑さを増す。「オプション」は実装優先度の意味であり、公開 API 一貫性を優先
- **トレードオフ**: コンパイル時の除外はできないが、バイナリサイズへの影響は微小

---

## リスクと軽減策

- **行ソルバの実装バグ** — 小規模パズル（1×1, 5×5）で網羅的テスト。既知の解答と照合
- **パフォーマンス要件（25×25 で 500ms）未達** — DP アルゴリズムの効率を検証。実装後にベンチマークで確認
- **バックトラッキングの組み合わせ爆発** — LinePropagator で前処理してから探索に入ることで探索空間を大幅に削減
- **ProbingSolver の複雑さ** — CspSolver 完成後に段階的に実装

---

## 参照

- [Nonogram solver - Rosetta Code](https://rosettacode.org/wiki/Nonogram_solver) — 各言語での実装例
- [Nonograms: Combinatorial questions and algorithms](https://www.sciencedirect.com/science/article/pii/S0166218X14000080) — 学術的計算量解析
- [Rust Error Handling Guide 2025](https://markaicode.com/rust-error-handling-2025-guide/) — エラーハンドリング ベストプラクティス
- [How To Solve Japanese Crosswords with Python, Rust, And WebAssembly](https://www.smartspate.com/how-to-solve-japanese-crosswords-with-python-rust-and-webassembly/) — Rust+WASM 実装事例
- `docs/naming-conventions.md` — 命名規則の詳細
- `docs/repository-plan.md` — リポジトリ全体の構成計画
