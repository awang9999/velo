// Graphical UI implementation (future)
// Requirement 1.1

use velo_app::App;

pub struct Gui {
    pub app: App,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            app: App::new(),
        }
    }
}

impl Default for Gui {
    fn default() -> Self {
        Self::new()
    }
}
