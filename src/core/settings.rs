use cosmic::app::Settings;

use crate::app::flags::Flags;

use super::i18n;

pub fn init() {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
}

pub fn settings() -> Settings {
    Settings::default()
        .size(cosmic::iced::Size::new(1024., 720.))
        .size_limits(cosmic::iced::Limits::new(
            cosmic::iced::Size::new(800., 400.),
            cosmic::iced::Size::INFINITY,
        ))
        .debug(false)
}

pub fn flags() -> Flags {
    Flags::default()
}
