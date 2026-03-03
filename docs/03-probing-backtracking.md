# 完全プロービング＋バックトラッキング（Fully Probing + Backtracking）

## 核心的アイディア

アーク整合性が不動点に達し、まだ未確定セルが残っている場合に用いる2段階の手法。

1. **完全プロービング（Fully Probing）**：実際に決定を確定させる前に、各未確定セルの両方の値（塗り・空白）を**試験的に**割り当て、アーク整合性を伝播させる。どちらかの値で即座に矛盾が発生するなら、もう一方が確定値となる。
2. **バックトラッキング**：プロービングでも確定できないセルが残った場合、1つのセルに値を仮定してスタックに状態を積む。矛盾が発生したら状態を戻し（バックトラック）、別の値を試みる。

プロービングは「賢い先読み」であり、バックトラッキングの探索木を大幅に削減する。

## アルゴリズムの特徴

- **LalaFrogKK**（Wu et al.）がこの手法を採用し、2011年以降のノノグラムソルバ大会で優勝を続けている
- プロービングはバックトラッキングの**前処理**として機能し、より多くのセルを確定することでバックトラッキングの深さを浅くする
- バックトラッキング中も各ノードでアーク整合性を伝播するため、枝刈り効果が高い
- 最悪計算量は指数オーダー（NP完全）だが、実用的なパズルでは十分に高速

## メリット

- 理論上、すべての正当なノノグラムパズルを解ける（完全性が保証される）
- プロービングにより、純粋なバックトラッキングより探索空間が大幅に小さくなる
- 矛盾を早期に検出することで、不要な探索を削除できる

## デメリット

- 実装が最も複雑（状態のスナップショット管理が必要）
- プロービング自体のコストが高い：未確定セル数を `u` とすると、各プロービングラウンドで最大 O(u × n × k) の計算が必要
- 困難なパズルでは依然として計算時間が長くなる可能性がある

## アルゴリズムの概要

全体の制御フローは以下の通り（アーク整合性を前提とする）。

```
function solve(grid):
    apply_arc_consistency(grid)
    if contradiction: return UNSAT
    if all cells determined: return SAT

    changed = fully_probe(grid)
    if changed: return solve(grid)  // 確定セルが増えたため再試行

    cell = select_cell(grid)        // 未確定セルを1つ選ぶ
    for value in [FILLED, EMPTY]:
        saved = snapshot(grid)
        grid[cell] = value
        result = solve(grid)
        if result == SAT: return SAT
        restore(grid, saved)        // バックトラック
    return UNSAT
```

### 完全プロービング（fully_probe）

```
function fully_probe(grid) -> changed:
    changed = false
    for each undetermined cell in grid:
        for value in [FILLED, EMPTY]:
            trial = snapshot(grid)
            trial[cell] = value
            apply_arc_consistency(trial)
            if contradiction:
                grid[cell] = opposite(value)  // 反対の値が確定
                apply_arc_consistency(grid)
                changed = true
                break
    return changed
```

> **注意**：プロービング中に新たな確定セルが生まれると、残りのセルに対するプロービング結果が変わる可能性がある。効率と正確さのトレードオフとして、一周ごとに再スキャンするか、差分更新するかを設計上選択する。

### セル選択のヒューリスティック

バックトラッキングが必要になった場合、どのセルに仮定を置くかが探索効率に大きく影響する。代表的なヒューリスティック：

| ヒューリスティック | 概要 |
|---|---|
| **MRV（最小残余値）** | 合法的配置数が最も少ないセルを選ぶ（制約が強いほど選ばれやすい） |
| **解数最小** | そのセルが属する行・列それぞれの合法的配置数の和が最小のセルを選ぶ |
| **中央バイアス** | グリッド中央付近のセルを優先（より多くの行・列に影響するため） |

### 状態のスナップショット

バックトラッキングを効率的に実現するには、グリッドの状態（各セルのドメイン）を保存・復元する仕組みが必要。

- **完全コピー**：実装シンプル、メモリ消費は O(n²) per ノード
- **差分記録（Undo スタック）**：変更箇所のみ記録し、バックトラック時に逆順に適用。メモリ効率が良い

## 参考文献

- Wu, C. C., Tsai, H. D., & Tsai, J. F. (2013). *An Efficient Approach to Solving Nonograms*. IEEE Transactions on Computational Intelligence and AI in Games, 5(3). https://ieeexplore.ieee.org/document/6476646/
- Batenburg, K. J., & Kosters, W. A. (2020). *On Efficiency of Fully Probing Mechanisms in Nonogram Solving Algorithm*. Advances in Computer Games. https://dl.acm.org/doi/10.1007/978-3-030-65883-0_10
