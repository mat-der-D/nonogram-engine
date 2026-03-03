use crate::error::SolveError;
use crate::solver::CellState;

/// Solve a single line (row or column) given its clues and current state.
///
/// Returns `Ok(true)` if any cell was updated, `Ok(false)` if no changes,
/// or `Err` if a contradiction is detected.
pub fn solve_line(clues: &[u8], line: &mut [CellState]) -> Result<bool, SolveError> {
    let k = clues.len();

    // Empty clues: all cells must be empty
    if k == 0 || (k == 1 && clues[0] == 0) {
        let mut changed = false;
        for cell in line.iter_mut() {
            if *cell == CellState::Filled {
                return Err(SolveError::InvalidProblem(
                    "contradiction: filled cell in empty-clue line".into(),
                ));
            }
            if *cell == CellState::Unknown {
                *cell = CellState::Empty;
                changed = true;
            }
        }
        return Ok(changed);
    }

    // Compute leftmost placement for each block
    let left = match compute_left(clues, line) {
        Some(l) => l,
        None => {
            return Err(SolveError::InvalidProblem(
                "contradiction: no valid left placement".into(),
            ));
        }
    };

    // Compute rightmost placement for each block
    let right = match compute_right(clues, line) {
        Some(r) => r,
        None => {
            return Err(SolveError::InvalidProblem(
                "contradiction: no valid right placement".into(),
            ));
        }
    };

    let mut changed = false;

    // Step 3: Overlap → mark FILLED
    for i in 0..k {
        let c = clues[i] as usize;
        let l_start = left[i];
        let r_start = right[i];

        // Overlap region: [r_start, l_start + c)
        if r_start < l_start + c {
            for cell in line[r_start..l_start + c].iter_mut() {
                if *cell == CellState::Unknown {
                    *cell = CellState::Filled;
                    changed = true;
                }
            }
        }
    }

    // Step 4: Cells not covered by any block's possible range → mark EMPTY
    // Blocks are ordered, so we can do a single linear pass.
    let mut block = 0;
    for (j, cell) in line.iter_mut().enumerate() {
        // Advance past blocks whose coverage ends before j
        while block < k && right[block] + clues[block] as usize <= j {
            block += 1;
        }
        if *cell == CellState::Unknown {
            let covered = block < k && j >= left[block];
            if !covered {
                *cell = CellState::Empty;
                changed = true;
            }
        }
    }

    Ok(changed)
}

/// Compute the leftmost valid start position for each block.
fn compute_left(clues: &[u8], line: &[CellState]) -> Option<Vec<usize>> {
    let n = line.len();
    let k = clues.len();
    let mut left = vec![0usize; k];

    let mut pos = 0;
    for i in 0..k {
        let c = clues[i] as usize;

        // Try to place block i starting at pos
        pos = find_left_placement(line, n, c, pos)?;
        left[i] = pos;

        // Next block must start after this block + 1 gap
        pos = pos + c + 1;
    }

    if has_trailing_filled(line, left[k - 1] + clues[k - 1] as usize) {
        return adjust_left_for_trailing_filled(clues, line, left);
    }

    Some(left)
}

/// Check if any FILLED cells exist at or after `from`.
fn has_trailing_filled(line: &[CellState], from: usize) -> bool {
    line[from..].contains(&CellState::Filled)
}

/// Try to place a block of size `c` starting at or after `start`.
/// Returns the valid start position, or None if impossible.
fn find_left_placement(line: &[CellState], n: usize, c: usize, mut start: usize) -> Option<usize> {
    loop {
        if start + c > n {
            return None;
        }

        // Check that all cells in [start..start+c] can be filled
        if let Some(offset) = line[start..start + c]
            .iter()
            .position(|cell| *cell == CellState::Empty)
        {
            start += offset + 1;
            continue;
        }

        // Check that cell after block (if exists) is not filled
        if start + c < n && line[start + c] == CellState::Filled {
            // Block would merge with next filled cell; slide right
            start += 1;
            continue;
        }

        return Some(start);
    }
}

/// Adjust left placements when there are filled cells after the last block.
/// Re-run placement with constraints from filled cells.
fn adjust_left_for_trailing_filled(
    clues: &[u8],
    line: &[CellState],
    mut left: Vec<usize>,
) -> Option<Vec<usize>> {
    let n = line.len();
    let k = clues.len();

    // Find the rightmost filled cell
    let rightmost_filled = line.iter().rposition(|c| *c == CellState::Filled)?;

    // The last block must cover rightmost_filled
    // So last block start <= rightmost_filled and start + c - 1 >= rightmost_filled
    let last_c = clues[k - 1] as usize;
    let min_start_for_last = (rightmost_filled + 1).saturating_sub(last_c);

    // Re-place from end: ensure last block covers the rightmost filled
    if left[k - 1] < min_start_for_last {
        left[k - 1] = find_left_placement(line, n, last_c, min_start_for_last)?;
    }

    // Propagate backwards: each block before must end before the next block starts
    for i in (0..k - 1).rev() {
        let c = clues[i] as usize;
        let max_end = left[i + 1] - 1; // must end before next block - 1 gap
        if left[i] + c > max_end {
            // Need to re-place this block
            let new_start = find_left_placement(line, n, c, left[i])?;
            if new_start + c > max_end {
                return None;
            }
            left[i] = new_start;
        }
    }

    // Propagate forwards to fix any issues
    for i in 1..k {
        let min_pos = left[i - 1] + clues[i - 1] as usize + 1;
        if left[i] < min_pos {
            left[i] = find_left_placement(line, n, clues[i] as usize, min_pos)?;
        }
    }

    // Final check: no filled cells after last block
    let last_end = left[k - 1] + clues[k - 1] as usize;
    if has_trailing_filled(line, last_end) {
        return None;
    }

    Some(left)
}

/// Compute the rightmost valid start position for each block.
fn compute_right(clues: &[u8], line: &[CellState]) -> Option<Vec<usize>> {
    let n = line.len();
    let k = clues.len();

    // Reverse the line and clues, compute left, then convert back
    let reversed_line: Vec<CellState> = line.iter().copied().rev().collect();
    let reversed_clues: Vec<u8> = clues.iter().copied().rev().collect();

    let reversed_left = compute_left(&reversed_clues, &reversed_line)?;

    // Convert reversed left positions back to right start positions
    let mut right = vec![0usize; k];
    for i in 0..k {
        let rev_i = k - 1 - i;
        let rev_start = reversed_left[rev_i];
        let c = clues[i] as usize;
        // In reversed line, block starts at rev_start and has length c
        // In original line, block ends at (n - 1 - rev_start) and starts at (n - rev_start - c)
        right[i] = n - rev_start - c;
    }

    Some(right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use CellState::*;

    #[test]
    fn test_empty_clues() {
        let mut line = vec![Unknown; 5];
        let changed = solve_line(&[], &mut line).unwrap();
        assert!(changed);
        assert!(line.iter().all(|c| *c == Empty));
    }

    #[test]
    fn test_zero_clue() {
        let mut line = vec![Unknown; 5];
        let changed = solve_line(&[0], &mut line).unwrap();
        assert!(changed);
        assert!(line.iter().all(|c| *c == Empty));
    }

    #[test]
    fn test_full_line() {
        let mut line = vec![Unknown; 5];
        let changed = solve_line(&[5], &mut line).unwrap();
        assert!(changed);
        assert!(line.iter().all(|c| *c == Filled));
    }

    #[test]
    fn test_overlap_center() {
        // [3] on length 5: leftmost [0,1,2], rightmost [2,3,4] → cell 2 is filled
        let mut line = vec![Unknown; 5];
        let changed = solve_line(&[3], &mut line).unwrap();
        assert!(changed);
        assert_eq!(line[2], Filled);
        // Cells 0,1 and 3,4 remain Unknown (could be either)
    }

    #[test]
    fn test_two_blocks() {
        // [2, 2] on length 5: only valid placement is [0,1] gap [2] [3,4]
        // All cells determined
        let mut line = vec![Unknown; 5];
        let changed = solve_line(&[2, 2], &mut line).unwrap();
        assert!(changed);
        assert_eq!(line, vec![Filled, Filled, Empty, Filled, Filled]);
    }

    #[test]
    fn test_with_existing_filled() {
        // [3] on length 5, cell 0 is already filled
        // Left: must start at 0 → [0,1,2]
        // Right: can start at 2 → [2,3,4]
        // Overlap: cell 2
        let mut line = vec![Filled, Unknown, Unknown, Unknown, Unknown];
        let changed = solve_line(&[3], &mut line).unwrap();
        assert!(changed);
        assert_eq!(line[2], Filled);
    }

    #[test]
    fn test_contradiction_filled_in_empty_clue() {
        let mut line = vec![Unknown, Filled, Unknown];
        let result = solve_line(&[], &mut line);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_change() {
        // Already solved line
        let mut line = vec![Filled, Filled, Empty, Filled, Filled];
        let changed = solve_line(&[2, 2], &mut line).unwrap();
        assert!(!changed);
    }

    #[test]
    fn test_single_cell_filled() {
        let mut line = vec![Unknown];
        let changed = solve_line(&[1], &mut line).unwrap();
        assert!(changed);
        assert_eq!(line[0], Filled);
    }

    #[test]
    fn test_single_cell_empty() {
        let mut line = vec![Unknown];
        let changed = solve_line(&[0], &mut line).unwrap();
        assert!(changed);
        assert_eq!(line[0], Empty);
    }
}
