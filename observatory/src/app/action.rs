use cosmic::widget::menu::Action as MenuAction;

use super::AppMessage;
use crate::app::ContextPage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    About,
    Settings,
}

impl MenuAction for Action {
    type Message = AppMessage;
    fn message(&self) -> Self::Message {
        match self {
            Action::About => AppMessage::ToggleContextPage(ContextPage::About),
            Action::Settings => AppMessage::ToggleContextPage(ContextPage::Settings),
        }
    }
}
