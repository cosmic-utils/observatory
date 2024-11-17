use crate::app::message::Message;

use cosmic::iced::Length;
use cosmic::{iced::widget, Element};

pub fn overview(_sys: &sysinfo::System) -> Element<Message> {
    Element::from(cosmic::widget::container(widget::text("Overview Page")).height(Length::Fill))
}
