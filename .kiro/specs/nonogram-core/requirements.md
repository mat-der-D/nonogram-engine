# 要件定義書

## はじめに

`nonogram-core` は2値（黒白）ノノグラムパズルのソルバエンジンを提供するRustライブラリクレートである。本クレートはフォーマット依存を持たず、パズルの内部表現・制約伝播・完全求解・解の判定を責務とする。

**設計思想の前提**:

- `Solver` トレイトは「入口のパズルから最終解答まで完全に求解する実体」を表す。部分解を返すことは許されない。`...Solver` サフィックスは `Solver` トレイトを実装する公開型にのみ使用する。
- 制約伝播（行列単位の確定マス計算）はすべての完全ソルバが内部で利用する共通処理であり、`Solver` トレイトを実装しない独立した内部コンポーネント（`LinePropagator`）として分離する。
- バックトラッキング（網羅的探索）は、解が確定しない限りあらゆる完全ソルバが最終フェーズとして必ず行う普遍的な手法であり、特定ソルバの名前に使うべきではない。バックトラッキングも共通内部コンポーネント（`Backtracker`）として分離する。
- 命名規則の詳細は `docs/naming-conventions.md` を参照すること。

---

## 要件

### 要件 1: 盤面・クルーのデータ表現

**目的:** ソルバ開発者として、ノノグラムパズルの盤面とクルー（ヒント）を型安全に表現したい。これにより、各ソルバが共通のデータ型を通じてパズル状態を参照・更新できるようにするため。

#### 受入基準

1. The nonogram-core shall provide a `Grid` type that represents an M×N board where each cell holds one of three states: `Unknown`, `Filled`, or `Blank`.
2. The nonogram-core shall provide a `Clue` type that represents an ordered sequence of one or more positive-integer block lengths for a single row or column.
3. The nonogram-core shall provide a `Puzzle` type that aggregates a `Grid`, a list of row `Clue`s, and a list of column `Clue`s.
4. If ノノグラムパズルの行数・列数・クルーリスト長が不整合である場合、the nonogram-core shall return an error when constructing a `Puzzle`.
5. The nonogram-core shall ensure that all core types (`Grid`, `Clue`, `Puzzle`) implement `Clone` and `Debug`.

---

### 要件 2: Solver トレイトと完全求解の保証

**目的:** ソルバアルゴリズム研究者として、統一されたインターフェースで複数の完全求解ソルバを追加・切り替えたい。これにより、異なるアルゴリズムを容易に比較できるようにするため。

#### 受入基準

1. The nonogram-core shall define a `Solver` trait with a method `solve(puzzle: &Puzzle) -> SolveResult`.
2. The nonogram-core shall define a `SolveResult` type that represents exactly one of three outcomes: `UniqueSolution(Grid)` (唯一解)、`MultipleSolutions(Vec<Grid>)` (複数解の代表例)、`NoSolution` (解なし).
3. When a type implements `Solver`, the nonogram-core shall guarantee that `solve` always returns a complete `SolveResult` and never returns a grid containing `Unknown` cells as a solution.
4. The nonogram-core shall allow any type implementing `Solver` to be used interchangeably via trait objects (`dyn Solver`).
5. The nonogram-core shall document all public items of the `Solver` trait and `SolveResult` type in American English.

---

### 要件 3: 制約伝播コンポーネント（LinePropagator）

**目的:** ソルバ開発者として、各行・各列を独立して解析する制約伝播を利用したい。これにより、確定マスを効率よく埋め、後段の網羅的探索コストを削減できるようにするため。

#### 受入基準

1. The nonogram-core shall provide a `LinePropagator` as a crate-internal component (not public API) that does NOT implement the `Solver` trait.
2. When a line（行または列）and its clue are given, the nonogram-core shall compute the intersection of all valid arrangements and mark cells that are `Filled` or `Blank` in every valid arrangement as definite.
3. While applying constraint propagation, the nonogram-core shall continue iterating across all rows and columns until a complete pass yields no newly determined cells.
4. If 制約伝播中に有効な配置が0件となる矛盾が検出された場合、the nonogram-core shall signal a contradiction to the calling solver component without returning a `SolveResult`.
5. The nonogram-core shall complete one full constraint propagation pass on a 25×25 puzzle within 500 milliseconds in a single-threaded debug build.

---

### 要件 4: CspSolver（CSP完全求解ソルバ）

**目的:** ソルバ利用者として、あらゆるノノグラムパズルを完全に解いてほしい。これにより、解なし・唯一解・複数解を正確に判定できるようにするため。

#### 受入基準

1. The nonogram-core shall provide a `CspSolver` that implements the `Solver` trait and fully solves any given puzzle.
2. When `CspSolver` begins solving, the nonogram-core shall first apply constraint propagation to reduce the search space.
3. If 制約伝播後に `Unknown` セルが残る場合、the nonogram-core shall exhaustively explore branches by hypothesizing `Filled` and `Blank` for each undetermined cell in turn.
4. If 分岐が矛盾に至る場合、the nonogram-core shall backtrack and try the alternative hypothesis.
5. When `CspSolver` finds a second distinct solution, the nonogram-core shall immediately halt the search and return `MultipleSolutions`.
6. If `CspSolver` exhausts all branches without finding a solution, the nonogram-core shall return `NoSolution`.
7. When `CspSolver` finds exactly one solution, the nonogram-core shall return `UniqueSolution(Grid)`.

---

### 要件 5: ProbingSolver（高度完全求解ソルバ、オプション）

**目的:** 上級ソルバ研究者として、網羅的探索の前に試行的制約伝播（プロービング）を行うことで探索コストを削減したい。これにより、特定のパズルクラスで高速な求解が可能になるようにするため。

#### 受入基準

1. Where ProbingSolver is included, the nonogram-core shall provide a `ProbingSolver` that implements the `Solver` trait and fully solves any given puzzle.
2. Where ProbingSolver is included, the nonogram-core shall first apply constraint propagation, then for each `Unknown` cell tentatively assign `Filled` and `Blank`, run constraint propagation on each hypothesis, and commit any cell value that is identical in both outcomes.
3. Where ProbingSolver is included, while プロービングで新たに確定するセルが存在する間, the nonogram-core shall continue the probing phase before proceeding to exhaustive search.
4. Where ProbingSolver is included, if プロービングで進展がなくなった場合, the nonogram-core shall proceed to exhaustive branch-and-backtrack search internally to complete the solution.
5. Where ProbingSolver is included, the nonogram-core shall return a `SolveResult` that reflects the complete search outcome (唯一解 / 複数解 / 解なし).

---

### 要件 6: 解の検証

**目的:** テスト作成者として、ソルバが返した解グリッドがクルーと矛盾しないことをプログラムで確認したい。これにより、ソルバの正確性を自動検証できるようにするため。

#### 受入基準

1. The nonogram-core shall provide a public `validate(puzzle: &Puzzle, grid: &Grid) -> bool` function that returns `true` if and only if every row and column in `grid` satisfies the corresponding clue in `puzzle`.
2. If `grid` にひとつでも `Unknown` セルが含まれる場合、the nonogram-core shall return `false` from `validate`.
3. The nonogram-core shall expose `validate` as a public API function callable by external crates.

---

### 要件 7: エラーハンドリング

**目的:** ライブラリ利用者として、不正な入力に対して明確なエラーを受け取りたい。これにより、呼び出し側でパニックなく安全にエラーを処理できるようにするため。

#### 受入基準

1. The nonogram-core shall define an `Error` enum that represents all possible error conditions of the library without panicking.
2. If `Puzzle` が不整合な次元（行数 ≠ 行クルー数、または 列数 ≠ 列クルー数）で構築された場合、the nonogram-core shall return `Err(Error::DimensionMismatch)`.
3. If `Clue` のブロック合計が対応する行・列の長さを超える場合、the nonogram-core shall return `Err(Error::ClueExceedsLength)`.
4. The nonogram-core shall not use `unwrap()` or `expect()` in non-test library code paths.

---

### 要件 8: 設計制約・品質基準

**目的:** プロジェクト設計者として、nonogram-core がフォーマット層に依存せず、十分なテストカバレッジを維持してほしい。これにより、ライブラリの独立性と信頼性を保てるようにするため。

#### 受入基準

1. The nonogram-core shall have no dependency on the `nonogram-format` crate in its `Cargo.toml`.
2. The nonogram-core shall achieve a line-coverage ratio of ≥ 80% as measured by `cargo-llvm-cov`.
3. The nonogram-core shall include unit tests within each source file under `#[cfg(test)]` modules.
4. The nonogram-core shall compile without warnings under `cargo build` with the Rust 2024 edition.
5. The nonogram-core shall document all public types, traits, functions, and methods in American English doc comments.
