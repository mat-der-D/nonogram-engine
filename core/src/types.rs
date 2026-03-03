#[derive(Debug, Clone)]
pub struct Problem {
    pub size: Size,
    pub clues: Clues,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: usize,
    height: usize,
}

#[derive(Debug, Clone)]
pub struct Clues {
    row: Vec<u8>,
    col: Vec<u8>,
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
    // 各要素は行を表す。u8 は bit-wise の塗り状態を表す。
    // 1 = filled, 0 = empty
    // 最初の要素が 1~8 マス目の塗り状態,
    // 次の要素が 9~16 マス目の塗り状態,...
    // となる。
    grid: Vec<Vec<u8>>,
}
