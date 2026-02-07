use core::iter;

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

    pub fn find(&self, position: Position) -> Option<(u16, u16, &Cell)> {
        use unicode_width::UnicodeWidthStr;

        const SEARCH_WIDTH: usize = 2;

        (0..SEARCH_WIDTH)
            .filter_map(|x_offset| x_offset.try_into().ok())
            .filter_map(|x_offset| position.x.checked_sub(x_offset))
            .filter_map(|x| {
                let key = CacheKey::new(x, position.y);
                let item = self.get(&key);
                item.map(|item| &item.0).and_then(|cell| {
                    cell.symbol().width().try_into().ok().and_then(|x_offset| {
                        x.checked_add(x_offset)
                            .is_some_and(|x| x > position.x)
                            .then_some((x, position.y, cell))
                    })
                })
            })
            .chain(iter::once((position.x, position.y, &Cell::EMPTY)))
            .next()
    }

    pub fn get(&self, key: &CacheKey) -> Option<&CacheItem> {
        self.0.get(key)
    }

    pub fn insert_or_replace(&mut self, key: CacheKey, cell: Cell) -> Option<CacheItem> {
        use unicode_width::UnicodeWidthStr;

        let width = cell.symbol().width();
        let item = CacheItem::new(cell);
        self.0.insert(key, item).inspect(|item| {
            for x in (item.0.symbol().width()..width)
                .filter_map(|x_offset| x_offset.try_into().ok())
                .filter_map(|x_offset| key.x.checked_add(x_offset))
            {
                let key = CacheKey { x, ..key };
                let _ = self.remove(&key);
            }
        })
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
