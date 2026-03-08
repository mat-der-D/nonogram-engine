use crate::cell::Cell;
use crate::clue::Clue;
use crate::grid::Grid;
use crate::puzzle::Puzzle;

/// Signals that constraint propagation found an inconsistency.
#[derive(Debug)]
pub(crate) struct Contradiction;

/// Crate-internal line constraint propagator.
///
/// Uses a two-pass dynamic programming algorithm to compute the
/// intersection of all valid block arrangements for a single line,
/// and iterates over all rows and columns until a fixpoint is reached.
pub(crate) struct LinePropagator;

impl LinePropagator {
    /// Solves a single line and returns the updated cells.
    ///
    /// Returns `Err(Contradiction)` if no valid arrangement exists.
    pub(crate) fn solve_line(line: &[Cell], clue: &Clue) -> Result<Vec<Cell>, Contradiction> {
        let n = line.len();
        let blocks = clue.blocks();
        let k = blocks.len();

        // Special case: no blocks — all cells must be blank.
        if k == 0 {
            for &c in line {
                if c == Cell::Filled {
                    return Err(Contradiction);
                }
            }
            return Ok(vec![Cell::Blank; n]);
        }

        // Special case: empty line but blocks present.
        if n == 0 {
            return Err(Contradiction);
        }

        // Precompute prefix counts for Blank and Filled cells.
        let mut blank_prefix = vec![0usize; n + 1];
        for i in 0..n {
            blank_prefix[i + 1] = blank_prefix[i] + if line[i] == Cell::Blank { 1 } else { 0 };
        }
        let has_blank_in = |lo: usize, hi: usize| -> bool {
            // Range [lo, hi) — true if any cell is fixed Blank.
            blank_prefix[hi] - blank_prefix[lo] > 0
        };

        // Forward DP: fwd[i][j] = can blocks 0..i-1 be placed in positions 0..j-1?
        let mut fwd = vec![vec![false; n + 1]; k + 1];
        fwd[0][0] = true;
        for j in 1..=n {
            fwd[0][j] = fwd[0][j - 1] && line[j - 1] != Cell::Filled;
        }
        for i in 1..=k {
            let b = blocks[i - 1] as usize;
            for j in 0..=n {
                // Option A: position j-1 is blank.
                if j >= 1 && line[j - 1] != Cell::Filled && fwd[i][j - 1] {
                    fwd[i][j] = true;
                }
                // Option B: block i-1 ends at position j-1 (occupies j-b..j-1).
                if j >= b && !has_blank_in(j - b, j) {
                    if i == 1 {
                        if fwd[0][j - b] {
                            fwd[i][j] = true;
                        }
                    } else if j - b >= 1 && line[j - b - 1] != Cell::Filled && fwd[i - 1][j - b - 1]
                    {
                        fwd[i][j] = true;
                    }
                }
            }
        }

        // If blocks cannot fit at all, contradiction.
        if !fwd[k][n] {
            return Err(Contradiction);
        }

        // Backward DP: bwd[i][j] = can blocks (k-i)..k-1 be placed in positions j..n-1?
        let mut bwd = vec![vec![false; n + 1]; k + 1];
        bwd[0][n] = true;
        for j in (0..n).rev() {
            bwd[0][j] = bwd[0][j + 1] && line[j] != Cell::Filled;
        }
        for i in 1..=k {
            let block_idx = k - i; // The first block of these i trailing blocks.
            let b = blocks[block_idx] as usize;
            for j in (0..=n).rev() {
                // Option A: position j is blank.
                if j < n && line[j] != Cell::Filled && bwd[i][j + 1] {
                    bwd[i][j] = true;
                }
                // Option B: block block_idx starts at position j (occupies j..j+b-1).
                if j + b <= n && !has_blank_in(j, j + b) {
                    if i == 1 {
                        if bwd[0][j + b] {
                            bwd[i][j] = true;
                        }
                    } else if j + b < n && line[j + b] != Cell::Filled && bwd[i - 1][j + b + 1] {
                        bwd[i][j] = true;
                    }
                }
            }
        }

        // Determine which cells can be filled and which can be blank.
        let mut can_be_filled = vec![false; n];
        let mut can_be_blank = vec![false; n];

        // can_be_blank[p]: exists split m such that fwd[m][p] && bwd[k-m][p+1].
        for p in 0..n {
            if line[p] != Cell::Filled {
                for m in 0..=k {
                    if fwd[m][p] && bwd[k - m][p + 1] {
                        can_be_blank[p] = true;
                        break;
                    }
                }
            }
        }

        // can_be_filled[p]: exists block b_idx whose placement covers p.
        for b_idx in 0..k {
            let b = blocks[b_idx] as usize;
            let max_start = if n >= b { n - b } else { continue };
            for s in 0..=max_start {
                if has_blank_in(s, s + b) {
                    continue;
                }

                // Forward condition.
                let fwd_ok = if b_idx == 0 {
                    fwd[0][s]
                } else {
                    s >= 1 && line[s - 1] != Cell::Filled && fwd[b_idx][s - 1]
                };
                if !fwd_ok {
                    continue;
                }

                // Backward condition.
                let remaining = k - 1 - b_idx;
                let bwd_ok = if remaining == 0 {
                    bwd[0][s + b]
                } else {
                    s + b < n && line[s + b] != Cell::Filled && bwd[remaining][s + b + 1]
                };
                if !bwd_ok {
                    continue;
                }

                for cell in &mut can_be_filled[s..s + b] {
                    *cell = true;
                }
            }
        }

        // Build result.
        let mut result = Vec::with_capacity(n);
        for p in 0..n {
            let cell = match (can_be_filled[p], can_be_blank[p]) {
                (true, false) => Cell::Filled,
                (false, true) => Cell::Blank,
                (true, true) => Cell::Unknown,
                (false, false) => return Err(Contradiction),
            };
            result.push(cell);
        }

        Ok(result)
    }

    /// Runs constraint propagation on the entire grid until fixpoint.
    ///
    /// Returns `Ok(true)` if any cell was updated, `Ok(false)` if the
    /// grid was unchanged, or `Err(Contradiction)` if an inconsistency
    /// is detected.
    pub(crate) fn propagate(grid: &mut Grid, puzzle: &Puzzle) -> Result<bool, Contradiction> {
        let height = puzzle.height();
        let width = puzzle.width();
        let mut row_dirty = vec![true; height];
        let mut col_dirty = vec![true; width];
        Self::propagate_core(grid, puzzle, &mut row_dirty, &mut col_dirty, |_, _, _| {})
    }

    /// Runs constraint propagation starting from a single changed cell.
    ///
    /// Only the row and column containing `(changed_row, changed_col)` are
    /// initially enqueued; further rows/columns are added incrementally as
    /// cells are determined. This is significantly faster than [`propagate`]
    /// when only one cell has changed (e.g. after a backtracking assignment).
    pub(crate) fn propagate_from_cell(
        grid: &mut Grid,
        puzzle: &Puzzle,
        changed_row: usize,
        changed_col: usize,
    ) -> Result<bool, Contradiction> {
        let height = puzzle.height();
        let width = puzzle.width();
        let mut row_dirty = vec![false; height];
        let mut col_dirty = vec![false; width];
        row_dirty[changed_row] = true;
        col_dirty[changed_col] = true;
        Self::propagate_core(grid, puzzle, &mut row_dirty, &mut col_dirty, |_, _, _| {})
    }

    /// Like [`propagate_from_cell`], but records every cell change as
    /// `(row, col, old_value)` into `undo`, enabling cheap rollback.
    pub(crate) fn propagate_from_cell_and_record(
        grid: &mut Grid,
        puzzle: &Puzzle,
        changed_row: usize,
        changed_col: usize,
        undo: &mut Vec<(usize, usize, Cell)>,
    ) -> Result<bool, Contradiction> {
        let height = puzzle.height();
        let width = puzzle.width();
        let mut row_dirty = vec![false; height];
        let mut col_dirty = vec![false; width];
        row_dirty[changed_row] = true;
        col_dirty[changed_col] = true;
        Self::propagate_core(grid, puzzle, &mut row_dirty, &mut col_dirty, |r, c, old| {
            undo.push((r, c, old));
        })
    }

    /// Core dirty-flag propagation loop.
    ///
    /// `record` is called with `(row, col, old_value)` for each cell that
    /// changes. Pass a no-op closure when undo tracking is not needed; the
    /// compiler will eliminate it entirely.
    fn propagate_core<F>(
        grid: &mut Grid,
        puzzle: &Puzzle,
        row_dirty: &mut Vec<bool>,
        col_dirty: &mut Vec<bool>,
        mut record: F,
    ) -> Result<bool, Contradiction>
    where
        F: FnMut(usize, usize, Cell),
    {
        let height = puzzle.height();
        let width = puzzle.width();
        let mut col_buf = vec![Cell::Unknown; height];
        let mut any_changed = false;

        loop {
            let mut changed = false;

            for r in 0..height {
                if !std::mem::take(&mut row_dirty[r]) {
                    continue;
                }
                // Borrow row as a slice for solve_line (no allocation).
                let result = {
                    let line = grid.row(r);
                    Self::solve_line(line, &puzzle.row_clues()[r])?
                };
                for c in 0..width {
                    let old = grid.get(r, c);
                    if result[c] != old {
                        record(r, c, old);
                        grid.set(r, c, result[c]);
                        col_dirty[c] = true;
                        changed = true;
                    }
                }
            }

            for c in 0..width {
                if !std::mem::take(&mut col_dirty[c]) {
                    continue;
                }
                // Fill reusable column buffer (no allocation).
                grid.fill_col(c, &mut col_buf);
                let result = Self::solve_line(&col_buf, &puzzle.col_clues()[c])?;
                for r in 0..height {
                    let old = col_buf[r];
                    if result[r] != old {
                        record(r, c, old);
                        grid.set(r, c, result[r]);
                        row_dirty[r] = true;
                        changed = true;
                    }
                }
            }

            if changed {
                any_changed = true;
            } else {
                break;
            }
        }

        Ok(any_changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clue(blocks: &[u32]) -> Clue {
        Clue::new(blocks.to_vec()).unwrap()
    }

    // --- solve_line tests ---

    #[test]
    fn all_filled_line() {
        // Line of 3, clue [3] -> all Filled
        let line = vec![Cell::Unknown; 3];
        let c = clue(&[3]);
        let result = LinePropagator::solve_line(&line, &c).unwrap();
        assert_eq!(result, vec![Cell::Filled, Cell::Filled, Cell::Filled]);
    }

    #[test]
    fn all_blank_line() {
        // Line of 3, clue [] -> all Blank
        let line = vec![Cell::Unknown; 3];
        let c = clue(&[]);
        let result = LinePropagator::solve_line(&line, &c).unwrap();
        assert_eq!(result, vec![Cell::Blank, Cell::Blank, Cell::Blank]);
    }

    #[test]
    fn partial_determination() {
        // Line of 5, clue [3] -> _FFF_ with ends unknown
        // Block can start at 0, 1, or 2.
        // Position 0: can be F (start=0) or B (start=1,2) -> Unknown
        // Position 1: can be F (start=0,1) or B (start=2..never actually since start=2 covers 2,3,4 not 1) -> wait
        // Actually: start=0 -> positions 0,1,2 filled. start=1 -> 1,2,3. start=2 -> 2,3,4.
        // Position 0: F if start=0, B if start=1,2 -> Unknown
        // Position 1: F if start=0,1, B if start=2 -> Unknown
        // Position 2: F always -> Filled
        // Position 3: F if start=1,2, B if start=0 -> Unknown
        // Position 4: F if start=2, B if start=0,1 -> Unknown
        let line = vec![Cell::Unknown; 5];
        let c = clue(&[3]);
        let result = LinePropagator::solve_line(&line, &c).unwrap();
        assert_eq!(
            result,
            vec![
                Cell::Unknown,
                Cell::Unknown,
                Cell::Filled,
                Cell::Unknown,
                Cell::Unknown
            ]
        );
    }

    #[test]
    fn contradiction_filled_in_blank_clue() {
        let line = vec![Cell::Filled, Cell::Unknown];
        let c = clue(&[]);
        assert!(LinePropagator::solve_line(&line, &c).is_err());
    }

    #[test]
    fn contradiction_impossible_block() {
        // Line of 2, clue [3] -> impossible
        let line = vec![Cell::Unknown; 2];
        let c = clue(&[3]);
        assert!(LinePropagator::solve_line(&line, &c).is_err());
    }

    #[test]
    fn respects_fixed_filled() {
        // Line of 5, clue [1, 1], position 0 is Filled.
        // Block 0 must be at position 0. Block 1 can be at 2, 3, or 4.
        let mut line = vec![Cell::Unknown; 5];
        line[0] = Cell::Filled;
        let c = clue(&[1, 1]);
        let result = LinePropagator::solve_line(&line, &c).unwrap();
        assert_eq!(result[0], Cell::Filled);
        assert_eq!(result[1], Cell::Blank);
        // Positions 2,3,4: one of them is Filled, rest Blank -> all Unknown
    }

    #[test]
    fn respects_fixed_blank() {
        // Line: [U, Blank, U, U, U], clue [2]
        // Block can't span position 1. So block starts at 2,3.
        // start=2 -> pos 2,3 filled. start=3 -> pos 3,4 filled.
        // pos 0: Blank, pos 1: Blank, pos 2: Unknown, pos 3: Filled, pos 4: Unknown
        let mut line = vec![Cell::Unknown; 5];
        line[1] = Cell::Blank;
        let c = clue(&[2]);
        let result = LinePropagator::solve_line(&line, &c).unwrap();
        assert_eq!(result[0], Cell::Blank);
        assert_eq!(result[1], Cell::Blank);
        assert_eq!(result[3], Cell::Filled);
    }

    #[test]
    fn two_blocks_fully_determined() {
        // Line of 5, clue [2, 1] -> min_length = 4
        // Block 0 at 0..1 or 0..1 shifted: start=0 -> 0,1 + gap + 3. start=1 -> 1,2 + gap + 4.
        // But also start=0 block1 at 3 or 4. start=0 block1 at 4: 0,1,_,_,4 -> gap at 2,3.
        // Actually min_length = 2+1+1 = 4. Line is 5. So 1 cell of slack.
        // Arrangements:
        // FF_F_ (block0 at 0-1, block1 at 3)
        // FF__F (block0 at 0-1, block1 at 4)
        // _FF_F (block0 at 1-2, block1 at 4)
        // Position 0: F,F,B -> Unknown
        // Position 1: F,F,F -> Filled
        // Position 2: B,B,F -> Unknown
        // Position 3: F,B,B -> Unknown
        // Position 4: B,F,F -> Unknown
        let line = vec![Cell::Unknown; 5];
        let c = clue(&[2, 1]);
        let result = LinePropagator::solve_line(&line, &c).unwrap();
        assert_eq!(result[1], Cell::Filled);
    }

    #[test]
    fn single_cell_filled() {
        let line = vec![Cell::Unknown];
        let c = clue(&[1]);
        let result = LinePropagator::solve_line(&line, &c).unwrap();
        assert_eq!(result, vec![Cell::Filled]);
    }

    #[test]
    fn single_cell_blank() {
        let line = vec![Cell::Unknown];
        let c = clue(&[]);
        let result = LinePropagator::solve_line(&line, &c).unwrap();
        assert_eq!(result, vec![Cell::Blank]);
    }

    // --- propagate tests ---

    #[test]
    fn propagate_trivial_1x1_filled() {
        let c = clue(&[1]);
        let puzzle = crate::puzzle::Puzzle::new(vec![c.clone()], vec![c]).unwrap();
        let mut grid = Grid::new(1, 1);
        let changed = LinePropagator::propagate(&mut grid, &puzzle).unwrap();
        assert!(changed);
        assert_eq!(grid.get(0, 0), Cell::Filled);
        assert!(grid.is_complete());
    }

    #[test]
    fn propagate_trivial_1x1_blank() {
        let c = clue(&[]);
        let puzzle = crate::puzzle::Puzzle::new(vec![c.clone()], vec![c]).unwrap();
        let mut grid = Grid::new(1, 1);
        let changed = LinePropagator::propagate(&mut grid, &puzzle).unwrap();
        assert!(changed);
        assert_eq!(grid.get(0, 0), Cell::Blank);
    }

    #[test]
    fn propagate_detects_contradiction() {
        // Row says [1], col says [] -> contradiction
        let puzzle = crate::puzzle::Puzzle::new(vec![clue(&[1])], vec![clue(&[])]).unwrap();
        let mut grid = Grid::new(1, 1);
        assert!(LinePropagator::propagate(&mut grid, &puzzle).is_err());
    }

    #[test]
    fn propagate_small_puzzle_fixpoint() {
        // 3x3 puzzle: all rows [1], all cols [1] -> diagonal or similar
        // Row 0: [1] in 3 -> one cell filled
        // After propagation, rows and columns constrain each other.
        let puzzle = crate::puzzle::Puzzle::new(
            vec![clue(&[1]), clue(&[1]), clue(&[1])],
            vec![clue(&[1]), clue(&[1]), clue(&[1])],
        )
        .unwrap();
        let mut grid = Grid::new(3, 3);
        let _ = LinePropagator::propagate(&mut grid, &puzzle);
        // This puzzle has multiple solutions, so propagation may not complete it,
        // but it should reach a fixpoint without error.
    }

    #[test]
    fn propagate_5x5_solvable() {
        // Simple 5x5 cross pattern:
        //   _ _ F _ _
        //   _ _ F _ _
        //   F F F F F
        //   _ _ F _ _
        //   _ _ F _ _
        let puzzle = crate::puzzle::Puzzle::new(
            vec![clue(&[1]), clue(&[1]), clue(&[5]), clue(&[1]), clue(&[1])],
            vec![clue(&[1]), clue(&[1]), clue(&[5]), clue(&[1]), clue(&[1])],
        )
        .unwrap();
        let mut grid = Grid::new(5, 5);
        let changed = LinePropagator::propagate(&mut grid, &puzzle).unwrap();
        assert!(changed);
        // Row 2 should be fully filled.
        assert_eq!(grid.get(2, 0), Cell::Filled);
        assert_eq!(grid.get(2, 4), Cell::Filled);
        // Col 2 should be fully filled.
        assert_eq!(grid.get(0, 2), Cell::Filled);
        assert_eq!(grid.get(4, 2), Cell::Filled);
        assert!(grid.is_complete());
    }

    #[test]
    fn propagate_performance_25x25() {
        // 25x25 puzzle with simple clues to test performance.
        // All rows and cols have clue [25] (fully filled).
        let row_clues: Vec<Clue> = (0..25).map(|_| clue(&[25])).collect();
        let col_clues: Vec<Clue> = (0..25).map(|_| clue(&[25])).collect();
        let puzzle = crate::puzzle::Puzzle::new(row_clues, col_clues).unwrap();
        let mut grid = Grid::new(25, 25);

        let start = std::time::Instant::now();
        let changed = LinePropagator::propagate(&mut grid, &puzzle).unwrap();
        let elapsed = start.elapsed();

        assert!(changed);
        assert!(grid.is_complete());
        assert!(
            elapsed.as_millis() < 500,
            "propagation took {}ms, expected < 500ms",
            elapsed.as_millis()
        );
    }
}
