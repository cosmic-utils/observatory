use crate::app::context::ContextPage;
use cosmic::iced::keyboard::{Key, Modifiers};

#[derive(Clone, Debug)]
pub enum AppMessage {
    SysInfoRefresh,
    KeyPressed(Key),

    OverviewApplicationSelect(Option<String>),
    OverviewApplicationClose,

    ProcessTermActive,
    ProcessKillActive,
    ProcessClick(Option<u32>),
    ProcessCategoryClick(u8),
    MulticoreView(bool),

    ResourceTabSelected(cosmic::widget::segmented_button::Entity),

    Key(Modifiers, Key),
    Modifiers(Modifiers),
    SystemThemeChanged,
    AppTheme(usize),
    Open(String),
    ToggleContextPage(ContextPage),
    ContextClose,
}
