mod app;
mod cosmic_theming;
mod widget;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = cosmic::app::Settings::default()
        .size(cosmic::iced::Size::new(1024., 768.));

    cosmic::app::run::<app::App>(settings, app::flags::Flags::default())?;

    Ok(())
}
