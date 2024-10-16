use crate::app::message::Message;

use cosmic::{iced::widget, Element};

pub fn overview(_sys: &sysinfo::System) -> Element<Message> {
    Element::from(widget::text("Overview Page"))
}
