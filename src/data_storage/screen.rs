#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coord {
    col: usize,
    row: usize,
}

impl Coord {
    pub fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }
    pub fn set_col(&mut self, col : usize) {
        self.col = col;
    }
    pub fn set_row(&mut self, row : usize) {
        self.row = row;
    }
    pub fn get_col(&self) -> usize {
        self.col
    }
    pub fn get_row(&self) -> usize {
        self.row
    }
}
