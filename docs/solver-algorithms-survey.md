# Nonogram Solver Algorithms: Survey

## Overview

Nonogram solving is NP-complete. No polynomial-time algorithm exists for the general case (unless P = NP).
This document surveys advanced solver algorithms beyond basic line solving, comparing their computational complexity and implementation difficulty.

---

## Algorithm Catalog

### 1. Line Solving (Constraint Propagation)

**Category:** Simple (already adopted as the baseline)

Line solving iterates over each row and column independently, applying logical rules to determine which cells can be definitively filled or left blank based on the clue alone.

Key technique: "leftmost/rightmost shift" — compute the leftmost valid placement and rightmost valid placement of all blocks; cells covered by both are definitely filled.

**Complexity (per line):**
- Naive: O(k * l²), where k = number of blocks in the clue, l = line length
- Optimized (dynamic programming): O(k * l), i.e., linear in grid area

**Standalone power:** Cannot solve all puzzles. Many nonograms require guessing.

**Implementation difficulty:** Low

---

### 2. Backtracking + Constraint Propagation (CSP approach)

**Category:** Standard advanced solver

Combines line solving with search. When line solving reaches a fixpoint (no more cells can be determined), the solver guesses the value of one undetermined cell, runs line solving again in that branch, and backtracks if a contradiction is found.

This is the standard approach used in most practical nonogram solvers.

**Complexity:**
- Worst case: exponential (NP-complete)
- In practice: most well-formed puzzles (especially those with a unique solution) are solved efficiently because constraint propagation prunes the search space aggressively

**Heuristics for cell selection:**
- Choose the most constrained cell (minimum remaining values, MRV)
- Choose the cell that causes the most propagation

**Implementation difficulty:** Medium
- Line solver must support partial state (cells can be FILLED, BLANK, or UNKNOWN)
- Requires state snapshot and rollback (either copying state or using an undo stack)
- Arc consistency (AC-3) can be layered on top for stronger propagation

---

### 3. Probing

**Category:** Advanced — tournament-winning approach

Probing is a technique that runs between line solving and backtracking. Before committing to a guess, the solver temporarily assumes each undetermined cell is FILLED (or BLANK), runs line solving in that hypothetical branch, and observes the result:

- If a contradiction is found: the cell must be the opposite value — this is a forced deduction.
- If no contradiction: the result informs which cell is the best candidate for backtracking.

The **LalaFrogKK** solver (Wu et al., 2011+), which has won multiple nonogram solving competitions, uses this three-phase structure:

```
Line solving → Fully Probing → Backtracking
```

**Probing variants:**
- **Simple probing:** Test one cell at a time; accept the first forced deduction found.
- **Full probing:** Test all undetermined cells before making a decision; use global information to pick the best guess.
- **Re-probing:** After a forced deduction from probing, run probing again before backtracking.

**Complexity:**
- Each probing pass: O(P * line_solve_cost), where P = number of undetermined cells
- Probing can eliminate many backtracking branches, so total work is often less than pure backtracking despite the higher per-step cost
- Worst case remains exponential

**Key tuning factors** (per research):
- Re-probing policy (when to probe again after a deduction)
- Probing sequence (order in which cells are tested)
- Computational overhead control (probing every cell is expensive; heuristics help)

**Implementation difficulty:** High
- Requires an efficient and restartable line solver
- State management for hypothetical branches
- Careful performance tuning to avoid probing becoming a bottleneck

---

### 4. Dancing Links (DLX)

**Category:** Exact cover approach

Donald Knuth's Algorithm X with Dancing Links is a well-known technique for exact cover problems (e.g., Sudoku, N-Queens). Nonograms can theoretically be encoded as exact cover problems.

However, the translation of a nonogram into an exact cover instance produces a matrix whose size grows very quickly — the number of possible block placements per row/column can be exponential in the clue length. This makes the DLX matrix unwieldy for larger puzzles.

**Complexity:**
- Encoding: O(2^k) possible positions per line in the worst case → matrix can be impractically large
- DLX itself: efficient for sparse exact cover instances, but nonogram encodings are rarely sparse enough

**Implementation difficulty:** High for encoding; DLX core itself is well-documented

**Assessment:** Not recommended as a primary solver. DLX excels at problems that map naturally to exact cover; nonogram translation is lossy and the resulting size is a known bottleneck. Research confirms this: *"the sizes of the translated problems are usually too large to solve efficiently."*

---

### 5. SAT Solver Encoding

**Category:** Reduction to SAT

Nonogram constraints can be encoded as a boolean CNF formula and passed to an off-the-shelf SAT solver (e.g., MiniSat, CaDiCaL).

**Encoding approach:**
- One boolean variable per cell: `x[i][j]` = true means filled
- For each row/column, generate clauses that exactly match the clue pattern
- Tseitin transformation converts the constraint formula into CNF with auxiliary variables

**Complexity:**
- Naive CNF encoding: potentially exponential clause count
- Optimized encoding (using auxiliary variables): polynomial in grid size, but with large constants
- SAT solving itself: exponential worst case, but modern CDCL solvers are extremely powerful in practice

**Pros:**
- Leverages decades of SAT solver research and highly optimized solvers
- Can produce all solutions or prove unsatisfiability rigorously
- Relatively little nonogram-specific code to write

**Cons:**
- External dependency on a SAT solver (or implementing one)
- Encoding overhead; difficult to extract meaningful partial solutions
- Overkill for small/medium puzzles; line solving + backtracking outperforms it there
- Research shows no method uniformly dominates: *"no method uniformly dominates; CSP and hybrid approaches lead to more compact encodings, though SAT can solve some of the hardest instances"*

**Implementation difficulty:** Medium (encoding logic) + High (if embedding a SAT solver in Rust without external bindings)

---

## Comparison Table

| Algorithm                        | Worst-case Complexity | Practical Performance  | Implementation Difficulty | Recommended? |
|----------------------------------|-----------------------|------------------------|---------------------------|--------------|
| Line Solving (DP)                | O(k * l) per line     | Excellent (limited)    | Low                        | Yes (base)  |
| Backtracking + Propagation (CSP) | Exponential           | Good on most puzzles   | Medium                     | Yes          |
| Probing (LalaFrogKK style)       | Exponential           | Best known in practice | High                       | Optional     |
| Dancing Links (DLX)              | Exponential + large encoding | Poor for nonograms | High                   | No           |
| SAT Solver Encoding              | Exponential           | Strong on hard cases   | Medium–High                | Optional     |

---

## Recommended Implementation Path

For this repository, the following phased approach is suggested:

### Phase 1: Line Solver (simple solver)
- O(k * l) dynamic programming per line
- Iterates until fixpoint
- Cannot solve all puzzles; documents its own limitations

### Phase 2: Backtracking + Constraint Propagation
- Extend Phase 1 with search
- MRV heuristic for cell selection
- Solves the vast majority of well-formed nonograms
- Manageable implementation complexity

### Phase 3: Probing (optional, "hardcore" solver)
- Add a probing layer between Phase 1 and Phase 2
- Significant complexity; may be pursued as a separate crate or feature flag
- Targets competition-grade performance

### Out of scope (for now): DLX, SAT encoding
- DLX is poorly suited to nonograms
- SAT encoding introduces a large external dependency; reconsider if Phase 3 proves insufficient

---

## References

- [Algorithms Effectiveness Comparison in Solving Nonogram Boards (ScienceDirect)](https://www.sciencedirect.com/science/article/pii/S1877050921016902)
- [An Efficient Algorithm for Solving Nonograms — Yu & Lee (Springer)](https://link.springer.com/article/10.1007/s10489-009-0200-0)
- [Nonogram — Wikipedia](https://en.wikipedia.org/wiki/Nonogram)
- [Complexity and Solvability of Nonogram Puzzles (University of Groningen thesis)](https://fse.studenttheses.ub.rug.nl/15287/1/Master_Educatie_2017_RAOosterman.pdf)
- [On Efficiency of Fully Probing Mechanisms in Nonogram Solving Algorithm (SpringerLink)](https://link.springer.com/chapter/10.1007/978-3-030-65883-0_10)
- [Exploring Effects of Fully Probing Sequence on Solving Nonogram Puzzles (ICGA Journal)](https://journals.sagepub.com/doi/10.3233/ICG-180069)
- [SAT-based Nonogram Solver — kbyte.io](https://www.kbyte.io/projects/201908_nonogram/)
- [Nonograms: Combinatorial Questions and Algorithms (ScienceDirect)](https://www.sciencedirect.com/science/article/pii/S0166218X14000080)
- [Solving Hard Instances of Nonograms (Medium / Smith-HCV)](https://medium.com/smith-hcv/solving-hard-instances-of-nonograms-35c68e4a26df)
- [The Magic of Nonogram Solving (UPC)](https://web.mat.upc.edu/victor.franco.sanchez/nonograms/nonograms/)
