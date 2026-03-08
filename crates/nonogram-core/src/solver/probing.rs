use crate::backtracker::Backtracker;
use crate::cell::Cell;
use crate::grid::Grid;
use crate::propagator::LinePropagator;
use crate::puzzle::Puzzle;
use crate::solver::{SolveResult, Solver};

/// A complete solver that uses probing to reduce the search space
/// before falling back to backtracking.
///
/// Probing tentatively assigns `Filled` and `Blank` to each unknown cell
/// and runs constraint propagation. If one hypothesis leads to a
/// contradiction, the opposite value is forced. If both are valid,
/// cells that agree in both outcomes are committed.
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

        // Phase 2: Probing loop.
        loop {
            let mut progress = false;

            let unknown_cells: Vec<(usize, usize)> = (0..grid.height())
                .flat_map(|r| (0..grid.width()).map(move |c| (r, c)))
                .filter(|&(r, c)| grid.get(r, c) == Cell::Unknown)
                .collect();

            for (r, c) in unknown_cells {
                if grid.get(r, c) != Cell::Unknown {
                    continue; // May have been determined by earlier probing in this pass.
                }

                // Try Filled.
                let filled_result = Self::probe(&grid, puzzle, r, c, Cell::Filled);
                // Try Blank.
                let blank_result = Self::probe(&grid, puzzle, r, c, Cell::Blank);

                match (filled_result, blank_result) {
                    (Err(_), Err(_)) => {
                        // Both contradict — puzzle has no solution.
                        return SolveResult::NoSolution;
                    }
                    (Err(_), Ok(_)) => {
                        // Filled contradicts — force Blank.
                        grid.set(r, c, Cell::Blank);
                        match LinePropagator::propagate_from_cell(&mut grid, puzzle, r, c) {
                            Ok(_) => {}
                            Err(_) => return SolveResult::NoSolution,
                        }
                        progress = true;
                    }
                    (Ok(_), Err(_)) => {
                        // Blank contradicts — force Filled.
                        grid.set(r, c, Cell::Filled);
                        match LinePropagator::propagate_from_cell(&mut grid, puzzle, r, c) {
                            Ok(_) => {}
                            Err(_) => return SolveResult::NoSolution,
                        }
                        progress = true;
                    }
                    (Ok(filled_grid), Ok(blank_grid)) => {
                        // Commit cells that agree in both outcomes.
                        let committed = Self::commit_common(&mut grid, &filled_grid, &blank_grid);
                        if committed {
                            match LinePropagator::propagate(&mut grid, puzzle) {
                                Ok(_) => {}
                                Err(_) => return SolveResult::NoSolution,
                            }
                            progress = true;
                        }
                    }
                }

                if grid.is_complete() {
                    return SolveResult::UniqueSolution(grid);
                }
            }

            if !progress {
                break;
            }
        }

        if grid.is_complete() {
            return SolveResult::UniqueSolution(grid);
        }

        // Phase 3: Fall back to backtracking.
        let solutions = Backtracker::search(&mut grid, puzzle, 2);
        SolveResult::from_solutions(solutions)
    }
}

impl ProbingSolver {
    /// Probes a cell by assigning a hypothesis and running propagation.
    fn probe(
        grid: &Grid,
        puzzle: &Puzzle,
        row: usize,
        col: usize,
        value: Cell,
    ) -> Result<Grid, ()> {
        let mut trial = grid.clone();
        trial.set(row, col, value);
        match LinePropagator::propagate_from_cell(&mut trial, puzzle, row, col) {
            Ok(_) => Ok(trial),
            Err(_) => Err(()),
        }
    }

    /// Commits cells that are identical in both grids but unknown in the
    /// main grid. Returns `true` if any cell was committed.
    fn commit_common(grid: &mut Grid, filled_grid: &Grid, blank_grid: &Grid) -> bool {
        let mut committed = false;
        for r in 0..grid.height() {
            for c in 0..grid.width() {
                if grid.get(r, c) == Cell::Unknown {
                    let f = filled_grid.get(r, c);
                    let b = blank_grid.get(r, c);
                    if f != Cell::Unknown && f == b {
                        grid.set(r, c, f);
                        committed = true;
                    }
                }
            }
        }
        committed
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

    fn solve_and_compare(puzzle: &Puzzle) {
        let csp_result = CspSolver.solve(puzzle);
        let probing_result = ProbingSolver.solve(puzzle);

        match (&csp_result, &probing_result) {
            (SolveResult::NoSolution, SolveResult::NoSolution) => {}
            (SolveResult::UniqueSolution(g1), SolveResult::UniqueSolution(g2)) => {
                assert_eq!(g1, g2, "unique solutions differ");
            }
            (SolveResult::MultipleSolutions(_), SolveResult::MultipleSolutions(_)) => {
                // Both found multiple solutions.
            }
            _ => {
                panic!("results differ: CspSolver={csp_result:?}, ProbingSolver={probing_result:?}")
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
