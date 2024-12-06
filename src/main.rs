use core::settings;

mod app;
mod core;
mod pages;
mod widgets;

fn main() -> cosmic::iced::Result {
    if flatpak_unsandbox::unsandbox(None).is_ok() {
        return Ok(())
    }
    settings::init();
    cosmic::app::run::<app::App>(settings::settings(), settings::flags())
}
