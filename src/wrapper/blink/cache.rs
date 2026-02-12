use core::fmt::Debug;
use core::mem;
use core::slice;

use alloc::vec::Vec;
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
pub struct CacheItem {
    pub key: CacheKey,
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub cell: Cell,
    pub tickstamp: usize,
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
    pub const fn new(key: CacheKey, cell: Cell, tickstamp: usize) -> Self {
        Self {
            key,
            cell,
            tickstamp,
        }
    }
}

impl Cache {
    pub const fn new() -> Self {
        Self(Vec::new())
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
            if let Some(cell) = item.map(|item| &item.cell)
                && let width = cell.symbol().width()
                && let Some(x_offset) = width.try_into().ok()
                && let Some(sum) = left.checked_add(x_offset)
                && sum > x
            {
                return Some((left, y, cell));
            }
        }

        None
    }

    pub fn get(&self, key: &CacheKey) -> Option<&CacheItem> {
        let Self(vec) = self;
        if let Ok(index) = vec.binary_search_by_key(key, |e| e.key) {
            let item = &vec[index];

            Some(item)
        } else {
            None
        }
    }

    pub fn insert_or_replace(
        &mut self,
        key: CacheKey,
        cell: Cell,
        tickstamp: usize,
    ) -> Option<CacheItem> {
        use unicode_width::UnicodeWidthStr;

        let Self(vec) = self;
        let CacheKey { x, y } = key;
        let end = cell.symbol().width();
        let item = match vec.binary_search_by_key(&key, |e| e.key) {
            Ok(index) => {
                let item = CacheItem::new(key, cell, tickstamp);
                let item = mem::replace(&mut vec[index], item);

                Some(item)
            }
            Err(index) => {
                let item = CacheItem::new(key, cell, tickstamp);
                vec.insert(index, item);

                None
            }
        };

        let start = item
            .as_ref()
            .map(|item| &item.cell)
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
        let Self(vec) = self;
        if let Ok(index) = vec.binary_search_by_key(key, |e| e.key) {
            let item = vec.remove(index);

            Some(item)
        } else {
            None
        }
    }

    pub fn retain(&mut self, f: impl Fn(CacheKey) -> bool) {
        self.0.retain(|item| f(item.key))
    }

    pub fn clear(&mut self) {
        self.0.clear();
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
