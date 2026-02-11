use ratatui_core::layout::Position;

use super::cache::Cache;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct State {
    pub cache: Cache,
    pub ticks: usize,
    pub blinking: bool,
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub cursor_position_changed: Option<Position>,
    pub cursor_hidden: bool,
    pub cursor_hidden_toggled: Option<()>,
    pub cursor_changed: bool,
}

impl State {
    pub const fn new() -> Self {
        Self {
            cache: Cache::new(),
            ticks: 0,
            blinking: false,
            cursor_position_changed: None,
            cursor_hidden: false,
            cursor_hidden_toggled: None,
            cursor_changed: true,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
