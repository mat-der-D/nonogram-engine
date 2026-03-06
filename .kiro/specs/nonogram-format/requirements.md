# 要件ドキュメント

## はじめに

`nonogram-format` は、ノノグラムパズルの問題・解答・テンプレートを JSON 形式で入出力するための型定義と変換ロジックを提供するクレートである。`nonogram-core` に依存するが、`nonogram-core` からは依存されない（依存方向を一方向に保つ）。

このクレートは `nonogram-wasm`・`apps/cli`・`apps/desktop` の全アプリケーション層から共通利用される。

---

## 要件

### Requirement 1: 問題 JSON のデシリアライズ

**目標:** アプリケーション開発者として、問題 JSON 文字列を `nonogram-core::Puzzle` に変換する手段が欲しい。これにより、JSON 入力からソルバに問題を渡せる。

#### 受け入れ基準

1. The `nonogram-format` shall `row_clues` と `col_clues` をフィールドに持つ問題 JSON を `nonogram-core::Puzzle` に変換できる関数を提供する。
2. When 問題 JSON が有効な形式で与えられたとき、the `nonogram-format` shall `row_clues`・`col_clues` それぞれを `Vec<Vec<u32>>` に正しく変換する。
3. When クルーが空配列（`[]`）のとき、the `nonogram-format` shall それをゼロブロックのクルーとして扱う。
4. If JSON に `row_clues` または `col_clues` フィールドが欠けているとき、the `nonogram-format` shall エラーを返す。
5. If JSON の数値が `u32` の範囲を超えるとき、the `nonogram-format` shall デシリアライズエラーを返す。

---

### Requirement 2: 解答 JSON のシリアライズ

**目標:** アプリケーション開発者として、ソルバの結果を解答 JSON 文字列に変換する手段が欲しい。これにより、UI や CLI で解答を正確に出力できる。

#### 受け入れ基準

1. The `nonogram-format` shall `nonogram-core::SolveResult::NoSolution` を `{"status": "none", "solutions": []}` にシリアライズする。
2. The `nonogram-format` shall `nonogram-core::SolveResult::UniqueSolution(grid)` を `{"status": "unique", "solutions": [[...]]}` にシリアライズする。
3. The `nonogram-format` shall `nonogram-core::SolveResult::MultipleSolutions(grids)` を `{"status": "multiple", "solutions": [[...], [...]]}` にシリアライズする。
4. When グリッドをシリアライズするとき、the `nonogram-format` shall `Cell::Filled` を `true`、`Cell::Blank` を `false` に変換する。
5. The `nonogram-format` shall グリッドを行優先順（`solutions[s][row][col]`）でシリアライズする。
6. While `Cell::Unknown` がグリッドに残っているとき、the `nonogram-format` shall その変換をエラーとして扱う（`Unknown` セルは完成した解答に含まれない）。

---

### Requirement 3: テンプレート JSON の生成

**目標:** アプリケーション開発者として、指定サイズの問題テンプレート JSON を生成する手段が欲しい。これにより、パズル作成の起点となる空の問題ファイルを出力できる。

#### 受け入れ基準

1. The `nonogram-format` shall 行数・列数を受け取り、全クルーを空配列（`[]`）に設定した問題テンプレート JSON を生成する関数を提供する。
2. When 行数 `N`・列数 `M` が指定されたとき、the `nonogram-format` shall `row_clues` に長さ `N` の配列、`col_clues` に長さ `M` の配列を持つ JSON を生成する。
3. The `nonogram-format` shall 生成されたテンプレートが有効な問題 JSON としてデシリアライズ可能な形式であることを保証する。

---

### Requirement 4: テストと品質保証

**目標:** 開発者として、`nonogram-format` の変換ロジックが単体テストで継続的に検証されることを期待する。

#### 受け入れ基準

1. The `nonogram-format` shall 正常な問題 JSON から `Puzzle` への変換を検証する単体テストを含む。
2. The `nonogram-format` shall 各 `SolveResult` バリアント（`NoSolution`・`UniqueSolution`・`MultipleSolutions`）の JSON シリアライズを検証する単体テストを含む。
3. The `nonogram-format` shall 不正な JSON 入力に対してエラーが返ることを検証する単体テストを含む。
4. The `nonogram-format` shall テンプレート生成の出力形式を検証する単体テストを含む。
5. The `nonogram-format` shall `cargo test --workspace` で全テストが通過する。
