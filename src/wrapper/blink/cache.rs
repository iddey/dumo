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

    pub fn iter(&self) -> slice::Iter<'_, CacheItem> {
        self.0.iter()
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
