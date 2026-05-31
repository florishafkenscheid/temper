#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FixedTime {
    completed_ticks: u64,
}

impl FixedTime {
    #[must_use]
    pub fn completed_ticks(&self) -> u64 {
        self.completed_ticks
    }

    pub(crate) fn advance(&mut self) {
        self.completed_ticks += 1;
    }
}
