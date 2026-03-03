#[derive(Debug, Clone)]
pub struct Problem {
    clues: Clues,
}

impl Problem {
    pub fn new(clues: Clues) -> Self {
        Self { clues }
    }

    pub fn clues(&self) -> &Clues {
        &self.clues
    }
}

#[derive(Debug, Clone)]
pub struct Clues {
    row: Vec<Vec<u8>>,
    col: Vec<Vec<u8>>,
}

impl Clues {
    pub fn new(row: Vec<Vec<u8>>, col: Vec<Vec<u8>>) -> Self {
        Self { row, col }
    }

    pub fn rows(&self) -> &[Vec<u8>] {
        &self.row
    }

    pub fn cols(&self) -> &[Vec<u8>] {
        &self.col
    }

    pub fn height(&self) -> usize {
        self.row.len()
    }

    pub fn width(&self) -> usize {
        self.col.len()
    }
}

#[derive(Debug, Clone)]
pub enum SolveResult {
    NoSolution,
    Unique(Solution),
    Multiple(Vec<Solution>),
}

#[derive(Debug, Clone)]
pub struct Solution {
    grid: Vec<Vec<bool>>,
}

impl Solution {
    pub fn new(grid: Vec<Vec<bool>>) -> Self {
        Self { grid }
    }

    pub fn grid(&self) -> &[Vec<bool>] {
        &self.grid
    }

    pub fn height(&self) -> usize {
        self.grid.len()
    }

    pub fn width(&self) -> usize {
        self.grid.first().map_or(0, |row| row.len())
    }
}
