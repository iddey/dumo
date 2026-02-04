use super::cache::Cache;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct State {
    pub cache: Cache,
    pub ticks: usize,
}

impl State {
    pub const fn new() -> Self {
        Self {
            cache: Cache::new(),
            ticks: 0,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
