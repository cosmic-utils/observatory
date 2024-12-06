use core::settings;

mod app;
mod core;
mod pages;
mod widgets;

fn main() -> cosmic::iced::Result {
    settings::init();
    let paths = std::fs::read_dir("/proc/");
    match paths {
        Ok(paths) => {
            for path in paths {
                log::warn!("{:?}", path);
            }
        }
        Err(e) => {
            log::error!("Failed to read /proc/: {}", e);
        }
    }
    cosmic::app::run::<app::App>(settings::settings(), settings::flags())
}
