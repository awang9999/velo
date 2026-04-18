// Terminal UI implementation
// Requirement 18

use velo_app::App;

pub struct Tui {
    pub app: App,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            app: App::new(),
        }
    }
}

impl Default for Tui {
    fn default() -> Self {
        Self::new()
    }
}
