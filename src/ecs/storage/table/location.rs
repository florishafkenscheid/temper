#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TableRowLocation {
    pub(crate) chunk: usize,
    pub(crate) row: usize,
}
