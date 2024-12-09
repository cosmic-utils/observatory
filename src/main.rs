mod app;
mod core;
mod pages;
mod style;
mod widgets;

use core::settings;

fn main() -> cosmic::iced::Result {
    settings::init();
    cosmic::app::run::<app::App>(settings::settings(), settings::flags())
}
