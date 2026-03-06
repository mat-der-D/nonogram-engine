use crate::error::Error;

/// Represents the clue (hint) for a single row or column of a nonogram puzzle.
///
/// A clue is an ordered sequence of block lengths. An empty sequence
/// represents a fully blank line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clue {
    blocks: Vec<u32>,
}

impl Clue {
    /// Creates a new clue from a sequence of block lengths.
    ///
    /// # Errors
    /// Returns `Error::InvalidBlockLength` if any block length is zero.
    pub fn new(blocks: Vec<u32>) -> Result<Self, Error> {
        for (i, &b) in blocks.iter().enumerate() {
            if b == 0 {
                return Err(Error::InvalidBlockLength { block_index: i });
            }
        }
        Ok(Self { blocks })
    }

    /// Returns the block lengths.
    pub fn blocks(&self) -> &[u32] {
        &self.blocks
    }

    /// Returns `true` if the clue has no blocks (fully blank line).
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Returns the minimum line length required to satisfy this clue.
    ///
    /// This is the sum of all block lengths plus the mandatory gaps
    /// (one cell) between consecutive blocks.
    pub fn min_length(&self) -> usize {
        if self.blocks.is_empty() {
            return 0;
        }
        let sum: usize = self.blocks.iter().map(|&b| b as usize).sum();
        sum + self.blocks.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_clue() {
        let c = Clue::new(vec![]).unwrap();
        assert!(c.is_empty());
        assert_eq!(c.blocks(), &[] as &[u32]);
        assert_eq!(c.min_length(), 0);
    }

    #[test]
    fn single_block() {
        let c = Clue::new(vec![3]).unwrap();
        assert!(!c.is_empty());
        assert_eq!(c.blocks(), &[3]);
        assert_eq!(c.min_length(), 3);
    }

    #[test]
    fn multiple_blocks() {
        let c = Clue::new(vec![2, 1, 3]).unwrap();
        assert_eq!(c.blocks(), &[2, 1, 3]);
        // min_length = 2 + 1 + 3 + 2 gaps = 8
        assert_eq!(c.min_length(), 8);
    }

    #[test]
    fn zero_block_length_error() {
        let result = Clue::new(vec![1, 0, 2]);
        assert_eq!(result, Err(Error::InvalidBlockLength { block_index: 1 }));
    }

    #[test]
    fn zero_block_length_at_start() {
        let result = Clue::new(vec![0]);
        assert_eq!(result, Err(Error::InvalidBlockLength { block_index: 0 }));
    }

    #[test]
    fn min_length_two_blocks() {
        let c = Clue::new(vec![1, 1]).unwrap();
        // 1 + 1 + 1 gap = 3
        assert_eq!(c.min_length(), 3);
    }
}
