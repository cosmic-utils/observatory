use cosmic::iced::keyboard::{Key, Modifiers};

use super::context_page::ContextPage;

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Refresh,
    KeyPressed(cosmic::iced_core::keyboard::Key),

    ProcessTermActive,
    ProcessKillActive,
    ProcessClick(Option<sysinfo::Pid>),
    ProcessCategoryClick(u8),

    ResourceTabSelected(cosmic::widget::segmented_button::Entity),

    Key(Modifiers, Key),
    Modifiers(Modifiers),
    SystemThemeChanged,
    AppTheme(usize),
    Open(String),
    ToggleContextPage(ContextPage),
    ContextClose,
}
