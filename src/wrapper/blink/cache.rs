use core::fmt::Debug;
use core::mem;
use core::slice;

use alloc::vec::Vec;
use ratatui_core::buffer::Cell;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CacheKey {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CacheItem {
    pub key: CacheKey,
    #[defmt(Debug2Format)]
    pub cell: Cell,
    pub changed: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Cache(Vec<CacheItem>);

impl CacheKey {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl CacheItem {
    pub const fn new(key: CacheKey, cell: Cell, changed: bool) -> Self {
        Self { key, cell, changed }
    }
}

impl Cache {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert_or_replace(&mut self, key: CacheKey, cell: Cell) -> Option<CacheItem> {
        use unicode_width::UnicodeWidthStr;

        let Self(vec) = self;
        match vec.binary_search_by_key(&key, |e| e.key) {
            Ok(index) => {
                let width = cell.symbol().width();
                let changed = cell != vec[index].cell;
                let item = CacheItem::new(key, cell, changed);
                let item = mem::replace(&mut vec[index], item);
                for x in (item.cell.symbol().width()..width)
                    .filter_map(|x_offset| x_offset.try_into().ok())
                    .filter_map(|x_offset| key.x.checked_add(x_offset))
                {
                    let key = CacheKey { x, ..key };
                    if let Ok(offset) = vec[index..].binary_search_by_key(&key, |e| e.key) {
                        let _ = vec.remove(index + offset);
                    }
                }

                Some(item)
            }
            Err(index) => {
                let item = CacheItem::new(key, cell, true);
                vec.insert(index, item);

                None
            }
        }
    }

    pub fn remove(&mut self, key: &CacheKey) -> Option<CacheItem> {
        let Self(vec) = self;
        if let Ok(index) = vec.binary_search_by_key(key, |e| e.key) {
            let item = vec.remove(index);

            Some(item)
        } else {
            None
        }
    }

    pub fn iter(&self) -> slice::Iter<'_, CacheItem> {
        self.0.iter()
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
