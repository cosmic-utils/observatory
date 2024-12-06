use core::settings;

mod app;
mod core;
mod pages;
mod widgets;

fn main() -> cosmic::iced::Result {
    let command = std::process::Command::new("flatpak-spawn").arg("observatory-daemon").output();
    if let Some(output) = command {
        println!("{}", String::from_utf8(output.stdout).unwrap());
    } else {
        println!("Could not flatpak-spawn!");
    }
    settings::init();
    cosmic::app::run::<app::App>(settings::settings(), settings::flags())
}
