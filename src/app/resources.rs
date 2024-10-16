use crate::app::message::Message;

use cosmic::{iced::widget, Element};

pub fn resources(_sys: &sysinfo::System) -> Element<Message> {
    Element::from(widget::text("Resources Page"))
}
