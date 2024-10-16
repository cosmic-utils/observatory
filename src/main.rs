mod app;
pub use app::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = vec![Page::Overview, Page::Resources, Page::Processes];

    let settings = Settings::default().size(Size::new(1024., 768.));

    cosmic::app::run::<App>(settings, input)?;

    Ok(())
}
