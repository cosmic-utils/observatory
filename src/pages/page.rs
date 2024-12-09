use crate::app::message::AppMessage;
use cosmic::app::context_drawer::ContextDrawer;
use cosmic::app::Task;
use cosmic::Element;

pub trait Page {
    // Required methods
    fn update(&mut self, message: AppMessage) -> Task<AppMessage>;

    fn view(&self) -> Element<AppMessage>;

    // Optional methods
    fn context_menu(&self) -> Option<ContextDrawer<AppMessage>> {
        None
    }

    fn footer(&self) -> Option<Element<AppMessage>> {
        None
    }
}
