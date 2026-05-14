/// Temporary application shell.
///
/// This exists to establish the libray boundary before the real app/plugin
/// runtime is implemented.
#[derive(Debug, Default)]
pub struct App;

impl App {
    pub fn new() -> Self {
        Self
    }

    pub fn run(self) {
        todo!("Real runtime loop comes later.")
    }
}
