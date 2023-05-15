use ncurses::{initscr, PANEL, WINDOW};

pub struct NcursesObjects {
    pub base_win: WINDOW,
    pub tree_win: WINDOW,
    pub message_border_win: WINDOW,
    pub message_win: WINDOW,

    pub base_panel: PANEL,
    pub tree_panel: PANEL,
    pub message_border_panel: PANEL,
    pub message_panel: PANEL,
}

impl Default for NcursesObjects {
    fn default() -> Self {
        Self {
            base_win: initscr(),
            tree_win: initscr(),
            message_border_win: initscr(),
            message_win: initscr(),
            base_panel: initscr(),
            tree_panel: initscr(),
            message_border_panel: initscr(),
            message_panel: initscr(),
        }
    }
}
