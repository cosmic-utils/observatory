pub trait Page {
    fn update(
        &mut self,
        sys: &sysinfo::System,
        message: crate::app::message::Message,
    ) -> cosmic::Task<cosmic::app::Message<crate::app::message::Message>>;

    fn context_menu(
        &self,
    ) -> Option<cosmic::app::context_drawer::ContextDrawer<'_, crate::app::message::Message>> {
        None
    }

    fn view(&self) -> cosmic::Element<'_, crate::app::message::Message>;

    fn footer(&self) -> Option<cosmic::Element<'_, crate::app::message::Message>> {
        None
    }
}
