pub mod processes;
pub mod resources;

use super::Message;
use cosmic::app::Task;
use cosmic::prelude::*;

pub trait Page {
    fn update(&mut self, _: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        cosmic::widget::horizontal_space().apply(Element::from)
    }

    fn footer(&self) -> Option<Element<Message>> {
        None
    }

    fn dialog(&self) -> Option<Element<Message>> {
        None
    }

    fn context_drawer(&self) -> Option<cosmic::app::context_drawer::ContextDrawer<Message>> {
        None
    }
}
