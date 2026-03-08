use crate::cell::Cell;
use crate::grid::Grid;
use crate::propagator::LinePropagator;
use crate::puzzle::Puzzle;

/// Crate-internal backtracking search engine.
///
/// Uses a degree heuristic to select the most constrained unknown cell,
/// then explores both hypotheses (`Filled` and `Blank`) with constraint
/// propagation after each assignment.
pub(crate) struct Backtracker;

impl Backtracker {
    /// Performs exhaustive search on the given grid.
    ///
    /// Uses an undo-log to avoid cloning the grid at each node; the grid is
    /// shared mutably across the recursion and rolled back after each branch.
    /// A clone is only made when a solution is actually found.
    ///
    /// Returns a vector of discovered solutions, containing at most
    /// `max_solutions` elements. Stops early once the limit is reached.
    pub(crate) fn search(grid: &mut Grid, puzzle: &Puzzle, max_solutions: usize) -> Vec<Grid> {
        if max_solutions == 0 {
            return Vec::new();
        }

        if grid.is_complete() {
            return vec![grid.clone()];
        }

        let (row, col) = match Self::select_cell(grid) {
            Some(pos) => pos,
            None => return Vec::new(),
        };

        let mut solutions = Vec::new();

        for &hypothesis in &[Cell::Filled, Cell::Blank] {
            // Record the cell we are about to overwrite, then apply hypothesis.
            let mut undo: Vec<(usize, usize, Cell)> = vec![(row, col, grid.get(row, col))];
            grid.set(row, col, hypothesis);

            if LinePropagator::propagate_from_cell_and_record(
                grid, puzzle, row, col, &mut undo,
            )
            .is_ok()
            {
                let remaining = max_solutions - solutions.len();
                let found = Self::search(grid, puzzle, remaining);
                solutions.extend(found);
            }

            // Restore grid to its state before this hypothesis.
            for &(r, c, old) in undo.iter().rev() {
                grid.set(r, c, old);
            }

            if solutions.len() >= max_solutions {
                solutions.truncate(max_solutions);
                return solutions;
            }
        }

        solutions
    }

    /// Selects an unknown cell using the degree heuristic.
    ///
    /// Picks the unknown cell whose row and column have the fewest
    /// total unknown cells (most constrained).
    fn select_cell(grid: &Grid) -> Option<(usize, usize)> {
        let row_counts: Vec<usize> = (0..grid.height())
            .map(|r| grid.row(r).iter().filter(|&&c| c == Cell::Unknown).count())
            .collect();

        let col_counts: Vec<usize> = (0..grid.width())
            .map(|c| grid.col(c).iter().filter(|&&c| c == Cell::Unknown).count())
            .collect();

        let mut best: Option<(usize, usize, usize)> = None;

        for (r, &rc) in row_counts.iter().enumerate() {
            for (c, &cc) in col_counts.iter().enumerate() {
                if grid.get(r, c) == Cell::Unknown {
                    let score = rc + cc;
                    let dominated = match best {
                        None => true,
                        Some((_, _, best_score)) => score < best_score,
                    };
                    if dominated {
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

    fn clue(blocks: &[u32]) -> Clue {
        Clue::new(blocks.to_vec()).unwrap()
    }

    #[test]
    fn unique_solution_puzzle() {
        // 2x2: rows [2], []; cols [1], [1]
        // Solution:
        //   F F
        //   B B
        let puzzle =
            crate::puzzle::Puzzle::new(vec![clue(&[2]), clue(&[])], vec![clue(&[1]), clue(&[1])])
                .unwrap();
        let mut grid = Grid::new(2, 2);
        let _ = LinePropagator::propagate(&mut grid, &puzzle);

        let solutions = Backtracker::search(&mut grid, &puzzle, 2);
        assert_eq!(solutions.len(), 1);

        let sol = &solutions[0];
        assert!(sol.is_complete());
        assert_eq!(sol.get(0, 0), Cell::Filled);
        assert_eq!(sol.get(0, 1), Cell::Filled);
        assert_eq!(sol.get(1, 0), Cell::Blank);
        assert_eq!(sol.get(1, 1), Cell::Blank);
    }

    #[test]
    fn multiple_solutions_stops_at_2() {
        // 2x2: rows [1], [1]; cols [1], [1]
        // Two solutions:
        //   F B    B F
        //   B F    F B
        let puzzle =
            crate::puzzle::Puzzle::new(vec![clue(&[1]), clue(&[1])], vec![clue(&[1]), clue(&[1])])
                .unwrap();
        let mut grid = Grid::new(2, 2);
        let _ = LinePropagator::propagate(&mut grid, &puzzle);

        let solutions = Backtracker::search(&mut grid, &puzzle, 2);
        assert_eq!(solutions.len(), 2);
        assert_ne!(solutions[0], solutions[1]);
    }

    #[test]
    fn no_solution_puzzle() {
        // 1x2: row [2]; cols [], []
        // Row wants both filled, but both columns want blank -> contradiction
        let puzzle =
            crate::puzzle::Puzzle::new(vec![clue(&[2])], vec![clue(&[]), clue(&[])]).unwrap();
        let mut grid = Grid::new(1, 2);

        // Propagation should detect contradiction, so search from initial grid.
        let solutions = Backtracker::search(&mut grid, &puzzle, 2);
        assert!(solutions.is_empty());
    }

    #[test]
    fn already_complete_grid() {
        let mut grid = Grid::new(1, 1);
        grid.set(0, 0, Cell::Filled);
        let puzzle = crate::puzzle::Puzzle::new(vec![clue(&[1])], vec![clue(&[1])]).unwrap();
        let solutions = Backtracker::search(&mut grid, &puzzle, 2);
        assert_eq!(solutions.len(), 1);
    }
}
