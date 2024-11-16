use core::localization;

mod app;
mod core;
mod widget;

fn main() -> cosmic::iced::Result {
    env_logger::init();

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    localization::init(&requested_languages);

    let settings = cosmic::app::Settings::default()
        .size(cosmic::iced::Size::new(1024., 720.))
        .size_limits(cosmic::iced::Limits::new(
            cosmic::iced::Size::new(800., 400.),
            cosmic::iced::Size::INFINITY,
        ))
        .debug(false);

    cosmic::app::run::<app::App>(settings, app::flags::Flags::default())
}
