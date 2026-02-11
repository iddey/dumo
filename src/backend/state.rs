use ratatui_core::layout::Position;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct State {
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub cursor_position: Position,
}

impl State {
    pub const fn new() -> Self {
        Self {
            cursor_position: Position::ORIGIN,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
