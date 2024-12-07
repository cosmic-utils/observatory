use cosmic::iced::keyboard::{Key, Modifiers};

use crate::app::context::ContextPage;

#[derive(Clone, Debug, PartialEq)]
pub enum AppMessage {
    Refresh,
    KeyPressed(Key),

    ApplicationSelect(Option<u32>),
    ApplicationClose,

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
