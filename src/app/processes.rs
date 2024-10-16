use crate::app::message::Message;

use cosmic::{iced::widget, Element};

pub fn processes(_sys: &sysinfo::System) -> Element<Message> {
    Element::from(widget::text("Processes Page"))
}
