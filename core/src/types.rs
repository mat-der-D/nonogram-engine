#[derive(Debug, Clone)]
pub struct Problem {
    size: Size,
    clues: Clues,
}

impl Problem {
    pub fn new(size: Size, clues: Clues) -> Self {
        Self { size, clues }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn clues(&self) -> &Clues {
        &self.clues
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: usize,
    height: usize,
}

impl Size {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
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
}

#[derive(Debug, Clone)]
pub enum SolveResult {
    NoSolution,
    Unique(Solution),
    Multiple(Vec<Solution>),
}

#[derive(Debug, Clone)]
pub struct Solution {
    size: Size,
    grid: Vec<Vec<bool>>,
}

impl Solution {
    pub fn new(size: Size, grid: Vec<Vec<bool>>) -> Self {
        Self { size, grid }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn grid(&self) -> &[Vec<bool>] {
        &self.grid
    }
}
