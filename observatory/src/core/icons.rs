// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub(crate) static ICON_CACHE: OnceLock<Mutex<IconCache>> = OnceLock::new();

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct IconCacheKey {
    name: String,
    size: u16,
}

pub struct IconCache {
    cache: HashMap<IconCacheKey, icon::Handle>,
}

impl IconCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();

        macro_rules! bundle {
            ($name:expr, $size:expr) => {
                let data: &'static [u8] =
                    include_bytes!(concat!("../../res/icons/bundled/", $name, ".svg"));
                cache.insert(
                    IconCacheKey {
                        name: String::from($name),
                        size: $size,
                    },
                    icon::from_svg_bytes(data).symbolic(true),
                );
            };
        }

        bundle!("harddisk-symbolic", 18);
        bundle!("memory-symbolic", 18);
        bundle!("processor-symbolic", 18);
        bundle!("speedometer-symbolic", 18);
        bundle!("user-home-symbolic", 18);
        bundle!("view-list-symbolic", 18);

        Self { cache }
    }

    pub fn get(&mut self, name: String, size: u16) -> icon::Icon {
        let handle = self
            .cache
            .entry(IconCacheKey {
                name: name.clone(),
                size,
            })
            .or_insert_with(|| icon::from_name(name).size(size).handle())
            .clone();
        icon::icon(handle).size(size)
    }

    pub fn get_handle(&mut self, name: String, size: u16) -> icon::Handle {
        let handle = self
            .cache
            .entry(IconCacheKey {
                name: name.clone(),
                size,
            })
            .or_insert_with(|| icon::from_name(name).size(size).handle())
            .clone();
        handle
    }
}

pub fn get_icon(name: String, size: u16) -> icon::Icon {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache.get(name, size)
}

#[allow(dead_code)]
pub fn get_handle(name: String, size: u16) -> icon::Handle {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache.get_handle(name, size)
}
