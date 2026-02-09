use alloc::collections::BTreeMap;
use alloc::collections::btree_map::Iter;
use ratatui_core::buffer::Cell;
use ratatui_core::layout::Position;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CacheKey {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CacheItem(#[defmt(Debug2Format)] pub Cell);

#[derive(Debug, Clone)]
pub struct Cache(BTreeMap<CacheKey, CacheItem>);

impl CacheKey {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl CacheItem {
    pub const fn new(cell: Cell) -> Self {
        Self(cell)
    }
}

impl Cache {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn find(&self, Position { x, y }: Position) -> Option<(u16, u16, &Cell)> {
        use unicode_width::UnicodeWidthStr;

        const MAX_END: usize = 2;

        for left in (0..MAX_END)
            .filter_map(|x_offset| x_offset.try_into().ok())
            .filter_map(|x_offset| x.checked_sub(x_offset))
        {
            let key = CacheKey::new(left, y);
            let item = self.get(&key);
            if let Some(cell) = item.map(|item| &item.0)
                && let width = cell.symbol().width()
                && let Some(x_offset) = width.try_into().ok()
                && let Some(sum) = left.checked_add(x_offset)
                && sum > x
            {
                return Some((left, y, cell));
            }
        }

        Some((x, y, &Cell::EMPTY))
    }

    pub fn get(&self, key: &CacheKey) -> Option<&CacheItem> {
        self.0.get(key)
    }

    pub fn insert_or_replace(&mut self, key: CacheKey, cell: Cell) -> Option<CacheItem> {
        use unicode_width::UnicodeWidthStr;

        let Self(map) = self;
        let CacheKey { x, y } = key;
        let end = cell.symbol().width();
        let item = CacheItem::new(cell);
        let item = map.insert(key, item);
        let start = item
            .as_ref()
            .map(|item| &item.0)
            .map(|cell| cell.symbol().width())
            .unwrap_or(1);

        for right in (start..end)
            .filter_map(|x_offset| x_offset.try_into().ok())
            .filter_map(|x_offset| x.checked_add(x_offset))
        {
            let key = CacheKey::new(right, y);
            let _ = self.remove(&key);
        }

        item
    }

    pub fn remove(&mut self, key: &CacheKey) -> Option<CacheItem> {
        self.0.remove(key)
    }

    pub fn iter(&self) -> Iter<'_, CacheKey, CacheItem> {
        self.0.iter()
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Cache {
    fn format(&self, formatter: defmt::Formatter) {
        defmt::write!(formatter, "{{");

        for (key, item) in self.iter() {
            defmt::write!(formatter, "{=?}: {=?}", key, item);
        }

        defmt::write!(formatter, "}}");
    }
}
