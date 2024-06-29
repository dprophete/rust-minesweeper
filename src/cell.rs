#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Hidden,
    Revealed(usize),
}
