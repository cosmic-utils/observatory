use core::settings;

mod app;
mod core;
mod pages;
mod widgets;

fn main() -> cosmic::iced::Result {
    settings::init();
    cosmic::app::run::<app::App>(settings::settings(), settings::flags())
}
