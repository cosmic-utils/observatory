use crate::system_info::SystemInfo;

pub trait Page {
    fn update(
        &mut self,
        sys: &SystemInfo,
        message: crate::app::message::AppMessage,
    ) -> cosmic::Task<cosmic::app::message::Message<crate::app::message::AppMessage>>;

    fn context_menu(
        &self,
    ) -> Option<cosmic::app::context_drawer::ContextDrawer<'_, crate::app::message::AppMessage>>
    {
        None
    }

    fn view(&self) -> cosmic::Element<'_, crate::app::message::AppMessage>;

    fn footer(&self) -> Option<cosmic::Element<'_, crate::app::message::AppMessage>> {
        None
    }
}
