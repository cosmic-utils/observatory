use std::sync::Mutex;

use cosmic::app::Settings;

use crate::app::flags::Flags;

use super::{
    i18n,
    icons::{IconCache, ICON_CACHE},
};

pub fn init() {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}

pub fn settings() -> Settings {
    Settings::default()
        .size(cosmic::iced::Size::new(1024., 720.))
        .size_limits(cosmic::iced::Limits::new(
            cosmic::iced::Size::new(800., 600.),
            cosmic::iced::Size::INFINITY,
        ))
        .debug(false)
}

pub fn flags() -> Flags {
    Flags::default()
}
