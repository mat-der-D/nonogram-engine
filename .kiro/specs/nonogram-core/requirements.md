# 要件定義書

## はじめに

`nonogram-core` は2値（黒白）ノノグラムパズルのソルバエンジンを提供するRustライブラリクレートである。本クレートはフォーマット依存を持たず、パズルの内部表現・制約伝播・バックトラッキングによる完全求解・解の判定を責務とする。トレイトベースの設計により、新しいソルバアルゴリズムを容易に追加できる拡張性を備える。

---

## 要件

### 要件 1: 盤面・クルーのデータ表現

**目的:** ソルバ開発者として、ノノグラムパズルの盤面とクルー（ヒント）を型安全に表現したい。これにより、各ソルバが共通のデータ型を通じてパズル状態を参照・更新できるようにするため。

#### 受入基準

1. The nonogram-core shall provide a `Grid` type that represents an M×N board where each cell holds one of three states: `Unknown`, `Filled`, or `Empty`.
2. The nonogram-core shall provide a `Clue` type that represents an ordered sequence of one or more positive-integer block lengths for a single row or column.
3. The nonogram-core shall provide a `Puzzle` type that aggregates a `Grid`, a list of row `Clue`s, and a list of column `Clue`s.
4. If ノノグラムパズルの行数・列数・クルーリスト長が不整合である場合、the nonogram-core shall return an error when constructing a `Puzzle`.
5. The nonogram-core shall ensure that all core types (`Grid`, `Clue`, `Puzzle`) implement `Clone` and `Debug`.

---

### 要件 2: Solver トレイト

**目的:** ソルバアルゴリズム研究者として、統一されたインターフェースで複数のソルバ実装を追加・切り替えたい。これにより、異なるアルゴリズムを容易に比較・合成できるようにするため。

#### 受入基準

1. The nonogram-core shall define a `Solver` trait with a method `solve(puzzle: &Puzzle) -> SolveResult`.
2. The nonogram-core shall define a `SolveResult` type that represents one of three outcomes: `Solved(Grid)` (唯一解)、`MultipleSolutions(Grid, Grid)` (複数解の代表例2件)、`NoSolution` (解なし).
3. The nonogram-core shall allow any type implementing `Solver` to be used interchangeably via trait objects (`dyn Solver`).
4. When a `Solver` implementation receives a `Puzzle`, the nonogram-core shall guarantee that the returned `SolveResult` is consistent with the puzzle's clue constraints.
5. The nonogram-core shall document all public items of the `Solver` trait and `SolveResult` type in American English.

---

### 要件 3: LineSolver（行列単位の制約伝播）

**目的:** ソルバ開発者として、各行・各列を独立して解析する制約伝播を利用したい。これにより、確定マスを効率よく埋め、後段のバックトラッキングコストを削減できるようにするため。

#### 受入基準

1. The nonogram-core shall provide a `LineSolver` that implements the `Solver` trait and operates by repeatedly applying constraint propagation to each row and column until no further progress is made.
2. When a line (行または列) and its clue are given, the nonogram-core shall compute the intersection of all valid arrangements and mark cells that are `Filled` or `Empty` in every arrangement as definite.
3. While constraint propagation is in progress, the nonogram-core shall continue iterating across all rows and columns until a complete pass yields no newly determined cells.
4. If constraint propagation alone cannot fully determine all cells, the nonogram-core shall return `Solved(Grid)` with the partially-filled grid when no contradiction is detected, or `NoSolution` if a contradiction is found.
5. The nonogram-core shall complete `LineSolver` on a 25×25 puzzle within 500 milliseconds in a single-threaded debug build.

---

### 要件 4: BacktrackingSolver（バックトラッキング＋制約伝播）

**目的:** ソルバ開発者として、LineSolver で解けないパズルも完全に解きたい。これにより、解なし・唯一解・複数解を正確に判定できるようにするため。

#### 受入基準

1. The nonogram-core shall provide a `BacktrackingSolver` that implements the `Solver` trait and uses constraint propagation as its inner reduction step before each branching decision.
2. When `BacktrackingSolver` encounters an `Unknown` cell, the nonogram-core shall branch by hypothesizing `Filled` and `Empty` in turn and recursively solving each branch.
3. If a branch produces a contradiction, the nonogram-core shall backtrack to the most recent branch point and try the alternative hypothesis.
4. When `BacktrackingSolver` finds a second distinct solution, the nonogram-core shall immediately halt the search and return `MultipleSolutions(solution1, solution2)`.
5. If `BacktrackingSolver` exhausts all branches without finding a solution, the nonogram-core shall return `NoSolution`.
6. When `BacktrackingSolver` finds exactly one solution, the nonogram-core shall return `Solved(Grid)`.

---

### 要件 5: ProbingSolver（プロービングソルバ、任意フェーズ）

**目的:** 上級ソルバ研究者として、バックトラッキングより先に試行的制約伝播（プロービング）を行うソルバを利用したい。これにより、特定パズルクラスでのバックトラック回数を削減できるようにするため。

#### 受入基準

1. Where ProbingSolver is included, the nonogram-core shall provide a `ProbingSolver` that implements the `Solver` trait.
2. Where ProbingSolver is included, the nonogram-core shall, for each `Unknown` cell, tentatively assign `Filled` and `Empty` and run constraint propagation, then commit any cell value that is identical in both branches.
3. Where ProbingSolver is included, the nonogram-core shall fall back to `BacktrackingSolver` after probing fails to make further progress.
4. Where ProbingSolver is included, the nonogram-core shall return a `SolveResult` consistent with the full search (唯一解 / 複数解 / 解なし).

---

### 要件 6: 解の検証

**目的:** テスト作成者として、ソルバが返した解グリッドがクルーと矛盾しないことをプログラムで確認したい。これにより、ソルバの正確性を自動検証できるようにするため。

#### 受入基準

1. The nonogram-core shall provide a `validate(puzzle: &Puzzle, grid: &Grid) -> bool` function that returns `true` if and only if every row and column in `grid` satisfies the corresponding clue in `puzzle`.
2. When a `Grid` contains any `Unknown` cell, the nonogram-core shall return `false` from `validate`.
3. The nonogram-core shall expose `validate` as a public API function.

---

### 要件 7: エラーハンドリング

**目的:** ライブラリ利用者として、不正な入力に対して明確なエラーを受け取りたい。これにより、呼び出し側でパニックなく安全にエラーを処理できるようにするため。

#### 受入基準

1. The nonogram-core shall define an `Error` type (または `enum`) that represents all possible error conditions of the library without panicking.
2. If a `Puzzle` is constructed with mismatched dimensions (行数 ≠ 行クルー数、または 列数 ≠ 列クルー数)、the nonogram-core shall return `Err(Error::DimensionMismatch)`.
3. If a `Clue` is constructed with a block sum exceeding the line length, the nonogram-core shall return `Err(Error::ClueExceedsLength)`.
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
