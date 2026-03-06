# 実装計画

## タスク一覧

- [x] 1. クレートの初期セットアップ
  - `nonogram-format` への依存を持たない `Cargo.toml` を設定し、外部依存ゼロ・Rust 2024 edition を指定する
  - `lib.rs` に全モジュール宣言（`cell`, `grid`, `clue`, `puzzle`, `error`, `validation`, `solver`, `propagator`, `backtracker`）と公開 API の `pub use` を記述する
  - `mod.rs` を使わない現代的モジュール構成（`solver.rs` + `solver/` ディレクトリ）を採用する
  - _Requirements: 8.1, 8.4_

- [x] 2. コアデータ型の実装

- [x] 2.1 (P) Cell enum の実装
  - `Unknown`, `Filled`, `Blank` の3バリアントを持つ enum を実装する
  - `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash` を derive する
  - `#[cfg(test)]` ブロックにバリアント値の等価性・Clone/Copy/Debug 動作を検証するテストを追加する
  - _Requirements: 1.1, 1.6_

- [x] 2.2 (P) Error / ValidationError enum の実装
  - `InvalidBlockLength`（block_index フィールド）、`ClueExceedsLength`（line_index, clue_min_length, line_length フィールド）、`EmptyClueList` の3バリアントを持つ `Error` enum を実装する
  - `DimensionMismatch`, `ContainsUnknown`, `ClueMismatch` の3バリアントを持つ `ValidationError` enum を実装する
  - `std::fmt::Display` と `std::error::Error` を両 enum に手動実装する
  - `Clone`, `Debug`, `PartialEq`, `Eq` を derive する
  - Display 出力の動作確認テストを追加する
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 2.3 Grid の実装（2.1 完了後）
  - セルを行優先の `Vec<Vec<Cell>>` で保持する `Grid` 構造体を実装する
  - `new`, `get`, `set`, `height`, `width`, `row`, `col`, `is_complete` のメソッドを実装する
  - `Clone`, `Debug`, `PartialEq`, `Eq` を derive する
  - ライブラリコード内で `unwrap`/`expect` を使用しない（境界外アクセスは panic の意図的なコントラクトとして設計書に明示済みのものを除く）
  - 構築・get/set・行列アクセス・is_complete のテストを追加する
  - _Requirements: 1.2, 1.6, 7.5_

- [x] 2.4 Clue の実装（2.2 完了後）
  - ブロック長列を保持する `Clue` 構造体を実装する
  - `new`（ゼロ長ブロックを `Error::InvalidBlockLength` で拒否）、`blocks`、`is_empty`、`min_length` のメソッドを実装する
  - `Clone`, `Debug`, `PartialEq`, `Eq` を derive する
  - 正常ケース・ゼロブロック長エラー・min_length 計算のテストを追加する
  - _Requirements: 1.3, 1.6, 1.7, 7.2_

- [x] 2.5 Puzzle の実装（2.4 完了後）
  - 行クルーと列クルーを所有する `Puzzle` 構造体を実装する
  - `new`（空クルーリストを `Error::EmptyClueList` で、超過クルーを `Error::ClueExceedsLength` で拒否）、`height`、`width`、`row_clues`、`col_clues` のメソッドを実装する
  - `Clone`, `Debug`, `PartialEq`, `Eq` を derive する
  - 正常構築・EmptyClueList エラー・ClueExceedsLength エラーのテストを追加する
  - _Requirements: 1.4, 1.5, 1.6, 7.3_

- [x] 3. (P) Solver トレイトと SolveResult の定義（Task 2 完了後）
  - `NoSolution`, `UniqueSolution(Grid)`, `MultipleSolutions(Vec<Grid>)` の3バリアントを持つ `SolveResult` enum を定義する
  - `solve(&self, puzzle: &Puzzle) -> SolveResult` メソッドを持つオブジェクト安全な `Solver` トレイトを定義する（`MultipleSolutions` のベクタは常に2要素以上という不変条件を doc comment に記載）
  - `solver.rs` でサブモジュール（`csp`, `probing`）の宣言を行う
  - 全公開型・トレイト・メソッドに American English のドキュメントコメントを付与する
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 4. (P) LinePropagator の実装（Task 2 完了後）

- [x] 4.1 単一ライン制約伝播（solve_line）の実装
  - 行または列と対応するクルーを受け取り、2パス DP（フォワード/バックワード）で全有効配置の交差集合を計算して確定セルを返す `solve_line` を実装する
  - フォワードパスで固定 Blank 位置へのブロック重複・固定 Filled 位置の空白扱いを不可能とマークし、有効配置が0件の場合は `Contradiction` を返す
  - 既確定の `Filled`/`Blank` セルに矛盾する配置を排除し、全配置で共通する値のみを確定セルとして返す
  - 全 Filled 行・全 Blank 行・部分確定行・矛盾行のテストを追加する
  - _Requirements: 3.1, 3.2_

- [x] 4.2 全盤面フィックスポイント反復（propagate）の実装
  - 全行・全列に `solve_line` を適用し、変化がなくなるまで反復する `propagate` を実装する
  - 矛盾検出時は即座に `Err(Contradiction)` を返す
  - 小パズルでフィックスポイント到達を確認するテストと、矛盾パズルの検出テストを追加する
  - 25×25 パズルの制約伝播が 500ms 以内であることをパフォーマンステストで確認する
  - _Requirements: 3.3, 3.4, 3.5_

- [x] 5. Backtracker の実装（Task 4 完了後）

- [x] 5.1 degree ヒューリスティックによるセル選択の実装
  - 各 Unknown セルについて「そのセルを含む行と列の Unknown セル数の合計」を計算し、最小値のセルを選択するヒューリスティックを実装する
  - `pub(crate)` で外部公開せず、`Solver` トレイトを実装しない `Backtracker` 構造体を定義する
  - _Requirements: 4.3, 4a.1_

- [x] 5.2 仮説設定・矛盾検出・ロールバックの探索ループの実装
  - `Grid::clone()` でスナップショットを保存し、Filled/Blank 各仮説に対して `LinePropagator::propagate` を実行し、矛盾時はスナップショットから復元する再帰探索ループを実装する
  - `max_solutions` に達した時点で早期停止し、収集済みの解ベクタを返す
  - 唯一解パズル・複数解パズル（2件で停止確認）・解なしパズルのテストを追加する
  - _Requirements: 4.4, 4.5, 4a.2, 4a.3, 4a.4, 4a.5_

- [x] 6. (P) CspSolver の実装（Tasks 3・4・5 完了後）
  - `Solver` トレイトを実装する `CspSolver` 構造体を実装する
  - `solve` の手順: 全 Unknown の Grid 生成 → `LinePropagator::propagate` → 完全なら `UniqueSolution` → 矛盾なら `NoSolution` → Unknown 残存なら `Backtracker::search(max_solutions=2)` → 解数に応じた `SolveResult` 返却
  - 返却する Grid に `Unknown` セルが含まれないことをテストで確認する
  - 1×1 パズル・5×5 典型パズル・解なしパズル・複数解パズルのテストを追加する
  - _Requirements: 2.3, 4.1, 4.2, 4.6, 4.7_

- [x] 7. (P) validate 関数の実装（Task 2 完了後）
  - パズルとグリッドを受け取り、クルーとの一致を検証する `validate` 関数を実装する
  - `DimensionMismatch` → `ContainsUnknown` → `ClueMismatch` の順でエラーを評価して返却する
  - 正解グリッド → Ok・クルー不一致 → ClueMismatch・Unknown 含有 → ContainsUnknown・サイズ不一致 → DimensionMismatch のテストを追加する
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 8. (P) ProbingSolver の実装（Tasks 3・4・5 完了後）
  - `Solver` トレイトを実装する `ProbingSolver` 構造体を実装する
  - `solve` の手順: 制約伝播 → プロービングループ（各 Unknown セルに Filled/Blank を仮定、矛盾側があれば逆値を強制代入して制約伝播を再実行、両仮定有効なら共通確定セルをコミット）→ 進展なしなら `Backtracker` へフォールバック → 完全な `SolveResult` 返却
  - 片側矛盾の強制代入（Filled 矛盾 → Blank 確定、Blank 矛盾 → Filled 確定）を正しく実装し、不要なバックトラッキングフォールバックを防ぐ
  - CspSolver と同一テストケースで結果が一致することを検証するテストを追加する
  - _Requirements: 2.3, 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 9. 結合テストと品質検証

- [x] 9.1 dyn Solver によるトレイトオブジェクト動作確認テストの追加
  - `dyn Solver` を通じて `CspSolver` と `ProbingSolver` を呼び出し、正常に `SolveResult` が返ることを確認する
  - _Requirements: 2.4_

- [x] 9.2 カバレッジ・コンパイル品質・ドキュメントの確認
  - `cargo test --workspace` が警告なしでパスすることを確認する
  - `cargo-llvm-cov` で行カバレッジ ≥ 80% を達成することを確認する
  - 全ソースファイルに `#[cfg(test)]` モジュールによる単体テストが含まれることを確認する
  - 全公開型・トレイト・関数・メソッドに American English doc コメントが付与されていることを確認する
  - _Requirements: 8.2, 8.3, 8.4, 8.5, 2.5_
