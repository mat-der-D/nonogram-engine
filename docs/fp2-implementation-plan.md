# FP2 実装作戦: Contrapositive 追跡 Probing

## 背景・動機

### 診断結果（30x30 usagi パズル）

```
総セル数: 900
Phase 1 (線解き)  : Unknown 772/900 (85.8%)  経過 2.8ms
Phase 2 (probing) : Unknown 758/900 (84.2%)  経過 622ms
  → forced=7, committed=5 (合計 14 セルのみ確定)
  → バックトラック必要: 2^758 ≈ 1.5e228 ノード
```

現行 `ProbingSolver`（FP1 相当）は probing に 622ms かけてたった 14 セルしか確定できない。
バックトラック空間が 2^758 になるため、バックトラッカーをどう改善しても解けない。

**根本原因**: FP1 は各セルを独立に probe するだけで、セル間の依存関係（対偶）を追跡しない。

---

## FP1 と FP2 の違い

### FP1（現行実装）

セル `i` を probe → 矛盾 or 一致 → 確定、以上。

```
probe(i = Filled) → grid_f
probe(i = Blank)  → grid_b

矛盾(Filled) → i = Blank 確定
矛盾(Blank)  → i = Filled 確定
両方成功     → grid_f と grid_b で一致するセルをコミット
```

セル間の「もし A なら B」という因果関係が捨てられる。

### FP2（実装目標）

probe 中に「A=Filled を仮定したら B=Filled が伝播した」という事実を記録する。

```
probe(i = Filled) → grid_f  かつ  i=Filled → j=v の関係を記録
probe(i = Blank)  → grid_b  かつ  i=Blank  → k=w の関係を記録
```

この記録から **対偶** を導出する：

```
i=Filled → j=Filled   ならば   j=Blank → i=Blank
i=Blank  → j=Blank    ならば   j=Filled → i=Filled
```

導出した対偶を次の probe の初期状態として注入することで、
FP1 では不可能な間接推論が可能になる。

---

## データ構造設計

### Implication（含意）

```rust
/// 「セル (row, col) が value になる」という含意の結論。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Literal {
    row: usize,
    col: usize,
    value: Cell, // Cell::Filled または Cell::Blank
}
```

### ImplicationGraph

```rust
/// FP2 の含意グラフ。
/// implications[lit] = lit が真であるとき、真になることが確定するリテラルの集合。
struct ImplicationGraph {
    // Literal → Vec<Literal> のマップ
    // (row, col, value) をキーにできるよう u32 に encode する等
    forward: HashMap<Literal, Vec<Literal>>,
}

impl ImplicationGraph {
    /// lit_a → lit_b の含意を追加し、対偶 ¬lit_b → ¬lit_a も同時に追加する。
    fn add(&mut self, from: Literal, to: Literal) {
        self.forward.entry(from).or_default().push(to);
        // 対偶: ¬to → ¬from
        let neg_to   = Literal { value: to.value.opposite(), ..to };
        let neg_from = Literal { value: from.value.opposite(), ..from };
        self.forward.entry(neg_to).or_default().push(neg_from);
    }

    /// lit が真になったとき、グラフ上で到達可能なリテラルをすべて返す。
    fn consequences(&self, lit: Literal) -> Vec<Literal> { /* BFS/DFS */ }
}
```

`Cell::opposite()`:

```rust
impl Cell {
    fn opposite(self) -> Self {
        match self {
            Cell::Filled => Cell::Blank,
            Cell::Blank => Cell::Filled,
            Cell::Unknown => panic!("Unknown has no opposite"),
        }
    }
}
```

---

## FP2 アルゴリズム（擬似コード）

Wu et al. 2013 の FP2 手順に準拠。

```
procedure FP2(grid, puzzle):
    graph = ImplicationGraph::new()

    // Phase 1: 線解き（既存の LinePropagator を流用）
    propagate(grid, puzzle)
    if contradiction or complete: return

    probe_queue = all unknown cells in grid  // 最初は全セル

    loop:
        if probe_queue is empty: break
        i = probe_queue.pop()
        if grid[i] != Unknown: continue

        // (A) i=Filled を仮定して probe
        grid_f = grid.clone()
        grid_f[i] = Filled
        result_f = propagate(grid_f, puzzle)
        if contradiction:
            force grid[i] = Blank
            propagate(grid, puzzle)
            continue

        // (B) i=Blank を仮定して probe
        grid_b = grid.clone()
        grid_b[i] = Blank
        result_b = propagate(grid_b, puzzle)
        if contradiction:
            force grid[i] = Filled
            propagate(grid, puzzle)
            continue

        // (C) 含意の記録
        for each cell j that changed in grid_f (Unknown → v):
            graph.add(Lit(i, Filled), Lit(j, v))
            // 対偶 Lit(j, ¬v) → Lit(i, Blank) も自動追加

        for each cell j that changed in grid_b (Unknown → v):
            graph.add(Lit(i, Blank), Lit(j, v))
            // 対偶 Lit(j, ¬v) → Lit(i, Filled) も自動追加

        // (D) 両 probe が一致するセルをコミット（FP1 の commit_common 相当）
        for each Unknown cell j in grid:
            if grid_f[j] == grid_b[j] != Unknown:
                grid[j] = grid_f[j]  // commit
                propagate_from_cell(grid, puzzle, j)
                // j の対偶含意から再 probe 対象を追加
                for lit in graph.consequences(Lit(j, grid[j].opposite())):
                    if grid[lit] == Unknown:
                        probe_queue.push(lit.cell)

    // Phase 3: バックトラック（既存 Backtracker を流用）
```

### 重要ポイント

- **対偶の伝播による再 probe キュー**:
  セル `j` が確定した瞬間、`graph.consequences(¬j)` から到達できるセルをキューに追加する。
  これが FP1（全セル再スキャン）より効率的な理由。

- **probe 時の含意記録**:
  `propagate_from_cell_and_record` が返す undo ログを再利用できる（old_value が Unknown だったセルが「新たに確定したセル」）。

---

## 既存コードとの統合

### 変更が必要なファイル

| ファイル | 変更内容 |
|---|---|
| `crates/nonogram-core/src/solver/probing.rs` | `ProbingSolver` を FP2 に置き換える（または `Fp2Solver` を追加） |
| `crates/nonogram-core/src/lib.rs` | 新 solver の pub export |
| `apps/cli/src/commands/solve.rs` | `--solver fp2` オプション追加（任意） |

### 再利用できる既存実装

- `LinePropagator::propagate_from_cell_and_record` — undo ログが「probe 中に確定したセルの記録」として使える
- `Backtracker::search` — Phase 3 はそのまま流用
- `ProbingSolver::probe` — 内部実装の参考にする（そのまま使うか修正）

### 追加が必要な実装

- `ImplicationGraph` 構造体（`probing.rs` 内 or 新ファイル `implication.rs`）
- `Cell::opposite()` メソッド（`cell.rs` に追加）
- FP2 メインループ

---

## テスト戦略

### ユニットテスト

1. **`ImplicationGraph::add` が対偶を正しく追加する**
2. **`ImplicationGraph::consequences` が BFS で正しく到達集合を返す**
3. **FP2 が FP1 より多くのセルを確定できる**（小パズルで検証）
4. **FP2 の結果が `CspSolver` と一致する**（既存の `solve_and_compare` パターン）

### 診断テスト（既存の `diagnose_30x30_phases` を拡張）

FP2 実装後、同じ診断テストを実行して：

- Phase 2 後の Unknown 数が 758 から有意に減少しているか確認
- 622ms → 速くなるか or 遅くなるか（per-probe コストが上がるが probe 回数が減る）

実行コマンド:
```
cargo test -p nonogram-core diagnose_30x30 -- --ignored --nocapture
```

---

## 期待される改善

Wu et al. 2013 の実験結果（25×25 パズル 1000 問）：

| 指標 | FP1 | FP2 |
|---|---|---|
| バックトラック呼び出し数 | 基準 | 約 30% 削減 |
| 処理時間 | 基準 | 約 19% 短縮 |
| FP 後の確定セル数 | ~201/625 | ~210/625 |

30×30 usagi については：
- **FP2 が効けば** Unknown 758 → 大幅減少し、バックトラック可能な範囲に入る可能性
- **FP2 でも不十分なら** Unknown が多すぎてバックトラックは依然不可能 → SAT/CDCL の検討へ

---

## 参考文献

- Wu et al. 2013, "An Efficient Approach to Solving Nonograms" — FP2 の擬似コードが掲載
  - NYCU機関リポジトリに全文PDF: https://ir.lib.nycu.edu.tw/bitstream/11536/22772/1/000324586300005.pdf
- `docs/solver-algorithms-survey.md` — アルゴリズム比較の概要
- `crates/nonogram-core/src/solver/probing.rs` — 現行 FP1 実装と診断テスト
