use crate::cell::Cell;
use crate::grid::Grid;
use crate::propagator::LinePropagator;
use crate::puzzle::Puzzle;
use crate::solver::{SolveResult, Solver};
use std::collections::VecDeque;

/// A complete solver that uses FP2 probing to reduce the search space, then
/// falls back to a probing-augmented backtracking search.
///
/// **Phase 2 (FP2)**: for each unknown cell, probes both hypotheses and records
/// implication edges ("if A=Filled then B=Filled") with automatic contrapositives.
/// When a cell is committed, the graph is traversed to directly determine further
/// cells and to focus subsequent probing.
///
/// **Phase 3 (probing search)**: instead of plain backtracking, each node runs an
/// FP1-style forced-cell pass before branching.  Contradictions are detected much
/// earlier, dramatically reducing the search tree for hard puzzles.
pub struct ProbingSolver;

impl Solver for ProbingSolver {
    fn solve(&self, puzzle: &Puzzle) -> SolveResult {
        let mut grid = Grid::new(puzzle.height(), puzzle.width());

        // Phase 1: Initial constraint propagation.
        match LinePropagator::propagate(&mut grid, puzzle) {
            Ok(_) => {}
            Err(_) => return SolveResult::NoSolution,
        }

        if grid.is_complete() {
            return SolveResult::UniqueSolution(grid);
        }

        // Phase 2: FP2 probing with implication graph.
        match Self::fp2_probe(&mut grid, puzzle) {
            Ok(()) => {}
            Err(()) => return SolveResult::NoSolution,
        }

        if grid.is_complete() {
            return SolveResult::UniqueSolution(grid);
        }

        // Phase 3: Backtracking with per-node forced-cell probing.
        let solutions = Self::probing_search(&mut grid, puzzle, 2);
        SolveResult::from_solutions(solutions)
    }
}

// ---------------------------------------------------------------------------
// Implication graph (FP2 data structure)
//
// A *literal* is encoded as `(row * width + col) * 2 + parity`
// where parity = 0 means Filled, parity = 1 means Blank.
// Negation of literal `k` is `k ^ 1`.
// ---------------------------------------------------------------------------

struct ImplicationGraph {
    forward: Vec<Vec<u32>>,
    width: usize,
}

impl ImplicationGraph {
    fn new(height: usize, width: usize) -> Self {
        ImplicationGraph {
            forward: vec![Vec::new(); height * width * 2],
            width,
        }
    }

    #[inline]
    fn encode(&self, row: usize, col: usize, value: Cell) -> usize {
        (row * self.width + col) * 2 + if value == Cell::Filled { 0 } else { 1 }
    }

    #[inline]
    fn decode(&self, idx: usize) -> (usize, usize, Cell) {
        let cell_idx = idx >> 1;
        let value = if idx & 1 == 0 { Cell::Filled } else { Cell::Blank };
        (cell_idx / self.width, cell_idx % self.width, value)
    }

    /// Adds `from → to` and its contrapositive `¬to → ¬from`.
    fn add(&mut self, from: usize, to: usize) {
        let fwd = &mut self.forward[from];
        if !fwd.contains(&(to as u32)) {
            fwd.push(to as u32);
        }
        let bwd = &mut self.forward[to ^ 1];
        if !bwd.contains(&((from ^ 1) as u32)) {
            bwd.push((from ^ 1) as u32);
        }
    }

    /// BFS: all literals reachable from `start` (excluding `start`).
    fn consequences(&self, start: usize) -> Vec<usize> {
        let n = self.forward.len();
        let mut visited = vec![false; n];
        visited[start] = true;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        let mut result = Vec::new();
        while let Some(current) = queue.pop_front() {
            for &next in &self.forward[current] {
                let next = next as usize;
                if !visited[next] {
                    visited[next] = true;
                    result.push(next);
                    queue.push_back(next);
                }
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Phase 2: FP2 probing
// ---------------------------------------------------------------------------

impl ProbingSolver {
    fn fp2_probe(grid: &mut Grid, puzzle: &Puzzle) -> Result<(), ()> {
        let height = puzzle.height();
        let width = puzzle.width();
        let mut graph = ImplicationGraph::new(height, width);

        let mut in_queue = vec![false; height * width];
        let mut probe_queue: VecDeque<(usize, usize)> = VecDeque::new();
        for r in 0..height {
            for c in 0..width {
                if grid.get(r, c) == Cell::Unknown {
                    in_queue[r * width + c] = true;
                    probe_queue.push_back((r, c));
                }
            }
        }

        while let Some((r, c)) = probe_queue.pop_front() {
            in_queue[r * width + c] = false;

            if grid.get(r, c) != Cell::Unknown {
                continue;
            }

            let filled_result = Self::probe_recording(grid, puzzle, r, c, Cell::Filled);
            let blank_result = Self::probe_recording(grid, puzzle, r, c, Cell::Blank);

            match (filled_result, blank_result) {
                (Err(_), Err(_)) => return Err(()),

                (Err(_), Ok((_, _))) => {
                    grid.set(r, c, Cell::Blank);
                    Self::commit_literal(
                        grid, puzzle, &graph, r, c, Cell::Blank, width,
                        &mut probe_queue, &mut in_queue,
                    )?;
                }

                (Ok((_, _)), Err(_)) => {
                    grid.set(r, c, Cell::Filled);
                    Self::commit_literal(
                        grid, puzzle, &graph, r, c, Cell::Filled, width,
                        &mut probe_queue, &mut in_queue,
                    )?;
                }

                (Ok((filled_grid, filled_changes)), Ok((blank_grid, blank_changes))) => {
                    let lit_filled = graph.encode(r, c, Cell::Filled);
                    let lit_blank = graph.encode(r, c, Cell::Blank);
                    for (jr, jc, jv) in &filled_changes {
                        graph.add(lit_filled, graph.encode(*jr, *jc, *jv));
                    }
                    for (jr, jc, jv) in &blank_changes {
                        graph.add(lit_blank, graph.encode(*jr, *jc, *jv));
                    }

                    let mut committed: Vec<(usize, usize, Cell)> = Vec::new();
                    for row2 in 0..height {
                        for col2 in 0..width {
                            if grid.get(row2, col2) == Cell::Unknown {
                                let f = filled_grid.get(row2, col2);
                                let b = blank_grid.get(row2, col2);
                                if f != Cell::Unknown && f == b {
                                    grid.set(row2, col2, f);
                                    committed.push((row2, col2, f));
                                }
                            }
                        }
                    }

                    if !committed.is_empty() {
                        match LinePropagator::propagate(grid, puzzle) {
                            Ok(_) => {}
                            Err(_) => return Err(()),
                        }
                        for (jr, jc, jv) in committed {
                            Self::commit_literal(
                                grid, puzzle, &graph, jr, jc, jv, width,
                                &mut probe_queue, &mut in_queue,
                            )?;
                        }
                    }
                }
            }

            if grid.is_complete() {
                return Ok(());
            }
        }

        Ok(())
    }

    /// After a cell is committed, propagates its implications through the graph
    /// and enqueues contrapositive consequences for targeted re-probing.
    fn commit_literal(
        grid: &mut Grid,
        puzzle: &Puzzle,
        graph: &ImplicationGraph,
        r: usize,
        c: usize,
        value: Cell,
        width: usize,
        probe_queue: &mut VecDeque<(usize, usize)>,
        in_queue: &mut Vec<bool>,
    ) -> Result<(), ()> {
        let lit = graph.encode(r, c, value);

        // Directly apply all forward-implied literals.
        for implied in graph.consequences(lit) {
            let (ir, ic, iv) = graph.decode(implied);
            let current = grid.get(ir, ic);
            if current == Cell::Unknown {
                grid.set(ir, ic, iv);
            } else if current != iv {
                return Err(());
            }
        }

        match LinePropagator::propagate(grid, puzzle) {
            Ok(_) => {}
            Err(_) => return Err(()),
        }

        // Enqueue cells reachable from the opposite literal for re-probing.
        for cons in graph.consequences(lit ^ 1) {
            let (cr, cc, _) = graph.decode(cons);
            if grid.get(cr, cc) == Cell::Unknown && !in_queue[cr * width + cc] {
                in_queue[cr * width + cc] = true;
                probe_queue.push_back((cr, cc));
            }
        }

        Ok(())
    }

    /// Probes a cell and records cells that changed from Unknown.
    fn probe_recording(
        grid: &Grid,
        puzzle: &Puzzle,
        row: usize,
        col: usize,
        value: Cell,
    ) -> Result<(Grid, Vec<(usize, usize, Cell)>), ()> {
        let mut trial = grid.clone();
        trial.set(row, col, value);
        let mut undo: Vec<(usize, usize, Cell)> = Vec::new();
        match LinePropagator::propagate_from_cell_and_record(
            &mut trial, puzzle, row, col, &mut undo,
        ) {
            Ok(_) => {
                let changed: Vec<(usize, usize, Cell)> = undo
                    .into_iter()
                    .filter(|(_, _, old)| *old == Cell::Unknown)
                    .map(|(r, c, _)| (r, c, trial.get(r, c)))
                    .collect();
                Ok((trial, changed))
            }
            Err(_) => Err(()),
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 3: Backtracking with per-node forced-cell probing
// ---------------------------------------------------------------------------

impl ProbingSolver {
    /// Entry point for the probing-augmented backtracking search.
    fn probing_search(grid: &mut Grid, puzzle: &Puzzle, max_solutions: usize) -> Vec<Grid> {
        if max_solutions == 0 {
            return Vec::new();
        }
        let mut solutions = Vec::new();
        Self::probing_search_inner(grid, puzzle, &mut solutions, max_solutions);
        solutions
    }

    fn probing_search_inner(
        grid: &mut Grid,
        puzzle: &Puzzle,
        solutions: &mut Vec<Grid>,
        max: usize,
    ) {
        if solutions.len() >= max {
            return;
        }

        // Forced-cell pass: for each unknown cell, probe both values.
        // If one leads to contradiction, force the other.
        // Record all changes in `node_undo` for rollback.
        let mut node_undo: Vec<(usize, usize, Cell)> = Vec::new();
        let forcing_ok = Self::force_cells(grid, puzzle, &mut node_undo);

        if forcing_ok.is_ok() {
            if grid.is_complete() {
                solutions.push(grid.clone());
            } else {
                // Select the most constrained unknown cell (degree heuristic).
                if let Some((row, col)) = Self::select_cell(grid) {
                    for &hypothesis in &[Cell::Filled, Cell::Blank] {
                        if solutions.len() >= max {
                            break;
                        }

                        let mut branch_undo: Vec<(usize, usize, Cell)> =
                            vec![(row, col, grid.get(row, col))];
                        grid.set(row, col, hypothesis);

                        if LinePropagator::propagate_from_cell_and_record(
                            grid, puzzle, row, col, &mut branch_undo,
                        )
                        .is_ok()
                        {
                            Self::probing_search_inner(grid, puzzle, solutions, max);
                        }

                        // Undo this branch's changes.
                        for &(r, c, old) in branch_undo.iter().rev() {
                            grid.set(r, c, old);
                        }
                    }
                }
            }
        }

        // Undo the forced-cell changes for this node.
        for &(r, c, old) in node_undo.iter().rev() {
            grid.set(r, c, old);
        }
    }

    /// Iterates over unknown cells; for each, probes both hypotheses.
    /// If one hypothesis leads to a contradiction, the other is forced.
    /// All changes (forced cells + their propagations) are recorded in `undo`.
    /// Returns `Err(())` if both hypotheses contradict (no solution).
    fn force_cells(
        grid: &mut Grid,
        puzzle: &Puzzle,
        undo: &mut Vec<(usize, usize, Cell)>,
    ) -> Result<(), ()> {
        loop {
            let mut progress = false;

            let cells: Vec<(usize, usize)> = (0..grid.height())
                .flat_map(|r| (0..grid.width()).map(move |c| (r, c)))
                .filter(|&(r, c)| grid.get(r, c) == Cell::Unknown)
                .collect();

            for (r, c) in cells {
                if grid.get(r, c) != Cell::Unknown {
                    continue;
                }

                let can_filled = Self::probe_simple(grid, puzzle, r, c, Cell::Filled);
                let can_blank = Self::probe_simple(grid, puzzle, r, c, Cell::Blank);

                match (can_filled, can_blank) {
                    (false, false) => return Err(()),
                    (true, false) => {
                        undo.push((r, c, Cell::Unknown));
                        grid.set(r, c, Cell::Filled);
                        LinePropagator::propagate_from_cell_and_record(
                            grid, puzzle, r, c, undo,
                        )
                        .map_err(|_| ())?;
                        progress = true;
                    }
                    (false, true) => {
                        undo.push((r, c, Cell::Unknown));
                        grid.set(r, c, Cell::Blank);
                        LinePropagator::propagate_from_cell_and_record(
                            grid, puzzle, r, c, undo,
                        )
                        .map_err(|_| ())?;
                        progress = true;
                    }
                    (true, true) => {} // Both valid, skip.
                }
            }

            if !progress {
                break;
            }
        }
        Ok(())
    }

    /// Probes a cell with the given hypothesis; returns `true` if consistent.
    fn probe_simple(grid: &Grid, puzzle: &Puzzle, row: usize, col: usize, value: Cell) -> bool {
        let mut trial = grid.clone();
        trial.set(row, col, value);
        LinePropagator::propagate_from_cell(&mut trial, puzzle, row, col).is_ok()
    }

    /// Degree heuristic: pick the unknown cell whose row + column has the
    /// fewest remaining unknowns (most constrained).
    fn select_cell(grid: &Grid) -> Option<(usize, usize)> {
        let row_counts: Vec<usize> = (0..grid.height())
            .map(|r| grid.row(r).iter().filter(|&&v| v == Cell::Unknown).count())
            .collect();
        let col_counts: Vec<usize> = (0..grid.width())
            .map(|c| grid.col(c).iter().filter(|&&v| v == Cell::Unknown).count())
            .collect();

        let mut best: Option<(usize, usize, usize)> = None;
        for r in 0..grid.height() {
            for c in 0..grid.width() {
                if grid.get(r, c) == Cell::Unknown {
                    let score = row_counts[r] + col_counts[c];
                    if best.map_or(true, |(_, _, s)| score < s) {
                        best = Some((r, c, score));
                    }
                }
            }
        }
        best.map(|(r, c, _)| (r, c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clue::Clue;
    use crate::solver::csp::CspSolver;

    fn clue(blocks: &[u32]) -> Clue {
        Clue::new(blocks.to_vec()).unwrap()
    }

    /// 診断テスト: 30x30 usagi パズルの各フェーズ後 Unknown セル数と経過時間を報告する。
    /// 実行: cargo test -p nonogram-core diagnose_30x30 -- --ignored --nocapture
    #[test]
    #[ignore]
    fn diagnose_30x30_phases() {
        use crate::grid::Grid;
        use crate::propagator::LinePropagator;

        let row_data: &[&[u32]] = &[
            &[], &[], &[4, 3], &[1, 1, 1], &[1, 1, 1], &[1, 1, 2, 1],
            &[1, 1, 1, 1], &[1, 1, 1, 1], &[1, 1, 1, 1], &[1, 1, 1],
            &[1, 2, 1], &[1, 2], &[1, 1], &[3, 1], &[1, 1, 1],
            &[2, 1, 1, 1], &[2, 4, 1], &[2, 5, 1], &[1, 2, 1], &[1, 1, 1],
            &[11, 1], &[11, 1], &[1, 1, 3], &[1, 1, 1, 1], &[1, 1, 1],
            &[1, 1, 1], &[1, 2, 6], &[2, 7], &[1, 2], &[17],
        ];
        let col_data: &[&[u32]] = &[
            &[], &[1], &[1, 1], &[2, 1], &[3, 3, 1], &[1, 3, 2, 1],
            &[1, 5, 1], &[2, 2, 1], &[1, 2, 1], &[1, 1, 2, 1], &[8, 2, 1],
            &[1, 2, 2, 1], &[1, 3, 2, 1], &[1, 1, 6, 1], &[1, 7, 2, 2, 1],
            &[1, 1, 1, 3, 1], &[1, 1, 5, 1], &[1, 4, 2, 1], &[1, 1, 1, 1],
            &[2, 1, 2], &[5, 1, 2], &[1, 1], &[1, 1], &[1],
            &[2, 1], &[2, 2, 1], &[6, 1], &[1, 1], &[4], &[],
        ];

        let row_clues: Vec<Clue> =
            row_data.iter().map(|b| Clue::new(b.to_vec()).unwrap()).collect();
        let col_clues: Vec<Clue> =
            col_data.iter().map(|b| Clue::new(b.to_vec()).unwrap()).collect();
        let puzzle = crate::puzzle::Puzzle::new(row_clues, col_clues).unwrap();

        let count_unknown = |g: &Grid| {
            (0..g.height())
                .flat_map(|r| (0..g.width()).map(move |c| (r, c)))
                .filter(|&(r, c)| g.get(r, c) == Cell::Unknown)
                .count()
        };

        let total = puzzle.height() * puzzle.width();
        eprintln!("=== 30x30 usagi 診断 (FP2 + probing search) ===");
        eprintln!("総セル数: {}", total);

        let mut grid = Grid::new(puzzle.height(), puzzle.width());

        // Phase 1
        let t1 = std::time::Instant::now();
        match LinePropagator::propagate(&mut grid, &puzzle) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Phase 1: 矛盾検出 → 解なし");
                return;
            }
        }
        let after_p1 = count_unknown(&grid);
        eprintln!(
            "Phase 1 (線解き)       : Unknown {}/{} ({:.1}%)  経過 {:?}",
            after_p1, total,
            after_p1 as f64 / total as f64 * 100.0,
            t1.elapsed()
        );

        if grid.is_complete() {
            eprintln!("→ 線解きのみで完全解決");
            return;
        }

        // Phase 2: FP2 probing
        let t2 = std::time::Instant::now();
        match ProbingSolver::fp2_probe(&mut grid, &puzzle) {
            Ok(()) => {}
            Err(()) => {
                eprintln!("Phase 2: 矛盾検出 → 解なし");
                return;
            }
        }
        let after_p2 = count_unknown(&grid);
        eprintln!(
            "Phase 2 (FP2)          : Unknown {}/{} ({:.1}%)  経過 {:?}",
            after_p2, total,
            after_p2 as f64 / total as f64 * 100.0,
            t2.elapsed()
        );

        if grid.is_complete() {
            eprintln!("→ FP2 で完全解決");
            return;
        }

        // Phase 3: Probing search
        let t3 = std::time::Instant::now();
        let solutions = ProbingSolver::probing_search(&mut grid, &puzzle, 2);
        eprintln!(
            "Phase 3 (probing search): {} 解  経過 {:?}",
            solutions.len(),
            t3.elapsed()
        );

        match solutions.len() {
            0 => eprintln!("→ 解なし"),
            1 => eprintln!("→ 唯一解"),
            _ => eprintln!("→ 複数解"),
        }

        eprintln!("総経過: {:?}", t1.elapsed());
    }

    fn solve_and_compare(puzzle: &Puzzle) {
        let csp_result = CspSolver.solve(puzzle);
        let probing_result = ProbingSolver.solve(puzzle);

        match (&csp_result, &probing_result) {
            (SolveResult::NoSolution, SolveResult::NoSolution) => {}
            (SolveResult::UniqueSolution(g1), SolveResult::UniqueSolution(g2)) => {
                assert_eq!(g1, g2, "unique solutions differ");
            }
            (SolveResult::MultipleSolutions(_), SolveResult::MultipleSolutions(_)) => {}
            _ => {
                panic!(
                    "results differ: CspSolver={csp_result:?}, ProbingSolver={probing_result:?}"
                )
            }
        }
    }

    #[test]
    fn solve_1x1_filled() {
        let puzzle = Puzzle::new(vec![clue(&[1])], vec![clue(&[1])]).unwrap();
        solve_and_compare(&puzzle);
    }

    #[test]
    fn solve_1x1_blank() {
        let puzzle = Puzzle::new(vec![clue(&[])], vec![clue(&[])]).unwrap();
        solve_and_compare(&puzzle);
    }

    #[test]
    fn solve_5x5_cross() {
        let puzzle = Puzzle::new(
            vec![clue(&[1]), clue(&[1]), clue(&[5]), clue(&[1]), clue(&[1])],
            vec![clue(&[1]), clue(&[1]), clue(&[5]), clue(&[1]), clue(&[1])],
        )
        .unwrap();
        solve_and_compare(&puzzle);
    }

    #[test]
    fn solve_no_solution() {
        let puzzle = Puzzle::new(vec![clue(&[2])], vec![clue(&[]), clue(&[])]).unwrap();
        solve_and_compare(&puzzle);
    }

    #[test]
    fn solve_multiple_solutions() {
        let puzzle =
            Puzzle::new(vec![clue(&[1]), clue(&[1])], vec![clue(&[1]), clue(&[1])]).unwrap();
        solve_and_compare(&puzzle);
    }

    #[test]
    fn probing_result_contains_no_unknown() {
        let puzzle = Puzzle::new(
            vec![clue(&[1, 1]), clue(&[1])],
            vec![clue(&[1]), clue(&[1]), clue(&[1])],
        )
        .unwrap();
        let result = ProbingSolver.solve(&puzzle);
        match result {
            SolveResult::UniqueSolution(grid) => {
                for r in 0..grid.height() {
                    for c in 0..grid.width() {
                        assert_ne!(grid.get(r, c), Cell::Unknown);
                    }
                }
            }
            SolveResult::MultipleSolutions(grids) => {
                for g in &grids {
                    for r in 0..g.height() {
                        for c in 0..g.width() {
                            assert_ne!(g.get(r, c), Cell::Unknown);
                        }
                    }
                }
            }
            SolveResult::NoSolution => {}
        }
    }
}
