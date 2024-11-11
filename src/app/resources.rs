use crate::app::message::Message;

use cosmic::{iced::{widget, Length}, Element};

pub fn resources(_sys: &sysinfo::System) -> Element<Message> {
    Element::from(cosmic::widget::container(widget::text("Resources Page")).height(Length::Fill))
}
