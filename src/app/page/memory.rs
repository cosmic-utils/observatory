use cosmic::{
    iced::{self, stream, Subscription},
    prelude::*,
    widget,
};
use futures_util::SinkExt;
use monitord_protocols::{
    monitord::{MemoryInfo, SnapshotRequest},
    protocols::MonitordServiceClient,
};
use std::collections::VecDeque;

use crate::{app::Message, fl};

/// Messages that are emitted relevant to the Memory Page
#[derive(Debug, Clone)]
pub enum MemoryMessage {
    Snapshot(MemoryInfo),
}

pub struct MemoryPage {
    memory_usage_history: VecDeque<f32>,
    memory_info: Option<MemoryInfo>,
}

impl MemoryPage {
    pub fn new() -> Self {
        Self {
            memory_usage_history: VecDeque::from(vec![0.0; 30]),
            memory_info: None,
        }
    }
}

impl super::Page for MemoryPage {
    fn update(&mut self, msg: Message) -> cosmic::app::Task<Message> {
        let tasks = Vec::new();
        match msg {
            Message::MemoryPage(MemoryMessage::Snapshot(snapshot)) => {
                self.memory_usage_history
                    .push_back(snapshot.memory_load_percent as f32 / 100.0);
                self.memory_usage_history.pop_front();

                self.memory_info = Some(snapshot);
            }
            _ => {}
        }

        cosmic::app::Task::batch(tasks)
    }

    fn view(&self) -> Element<Message> {
        if let Some(memory_info) = &self.memory_info {
            let theme = cosmic::theme::active();
            let cosmic = theme.cosmic();
            widget::row()
                .spacing(cosmic.space_xxs())
                .push(
                    widget::canvas(crate::widget::graph::LineGraph {
                        points: self.memory_usage_history.iter().cloned().collect(),
                    })
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill),
                )
                .push(
                    widget::settings::view_column(vec![
                        widget::settings::section()
                            .title(fl!("memory-info"))
                            .add(widget::settings::item(
                                fl!("total-memory"),
                                memory_info
                                    .total_memory_bytes
                                    .apply(crate::helpers::get_bytes)
                                    .apply(widget::text::body),
                            ))
                            .add(widget::settings::item(
                                fl!("total-swap"),
                                memory_info
                                    .swap_total_bytes
                                    .apply(crate::helpers::get_bytes)
                                    .apply(widget::text::body),
                            ))
                            .add_maybe(memory_info.dram_info.as_ref().map(|i| {
                                widget::settings::item(
                                    fl!("dram-frequency"),
                                    format!("{} MHz", i.frequency_mhz).apply(widget::text::body),
                                )
                            }))
                            .add_maybe(memory_info.dram_info.as_ref().map(|i| {
                                widget::settings::item(
                                    fl!("dram-type"),
                                    i.memory_type.clone().apply(widget::text::body),
                                )
                            }))
                            .apply(Element::from),
                        widget::settings::section()
                            .title(fl!("memory-stats"))
                            .add(widget::settings::item(
                                fl!("used-memory"),
                                memory_info
                                    .used_memory_bytes
                                    .apply(crate::helpers::get_bytes)
                                    .apply(widget::text::body),
                            ))
                            .add(widget::settings::item(
                                fl!("used-swap"),
                                memory_info
                                    .swap_used_bytes
                                    .apply(crate::helpers::get_bytes)
                                    .apply(widget::text::body),
                            ))
                            .apply(Element::from),
                    ])
                    .apply(widget::scrollable),
                )
                .apply(Element::from)
        } else {
            widget::horizontal_space().apply(Element::from)
        }
    }

    fn subscription(&self) -> Vec<Subscription<Message>> {
        vec![Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let mut service = MonitordServiceClient::connect("http://127.0.0.1:50051")
                    .await
                    .unwrap();

                let request = tonic::Request::new(SnapshotRequest { interval_ms: 1000 });

                let mut stream = service
                    .stream_memory_info(request)
                    .await
                    .unwrap()
                    .into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(message) = message {
                        sender
                            .send(Message::MemoryPage(MemoryMessage::Snapshot(message)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
