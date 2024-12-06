use core::settings;

mod app;
mod core;
mod pages;
mod widgets;

fn main() -> cosmic::iced::Result {
    settings::init();
    log::warn!("{}", std::str::from_utf8(std::process::Command::new("flatpak-spawn").arg("--host").arg("\"ps -e\"").output().unwrap().stdout.as_slice()).unwrap());
    cosmic::app::run::<app::App>(settings::settings(), settings::flags())
}
