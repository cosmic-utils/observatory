mod app;
mod widget;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = cosmic::app::Settings::default()
        .size(cosmic::iced::Size::new(1024., 720.))
        .size_limits(cosmic::iced::Limits::new(cosmic::iced::Size::new(800., 400.), cosmic::iced::Size::INFINITY))
        .debug(false);

    cosmic::app::run::<app::App>(settings, app::flags::Flags::default())?;

    Ok(())
}
