use std::collections::VecDeque;

use crate::{
    app::Message,
    fl,
    helpers::{format_number, get_bytes},
};
use cosmic::{
    iced::{self, stream, Subscription},
    prelude::*,
    widget,
};
use futures_util::SinkExt;
use monitord_protocols::{
    monitord::{CpuInfo, SnapshotRequest},
    protocols::MonitordServiceClient,
};

/// Messages that are emitted that are relevant to the CPU page
#[derive(Debug, Clone)]
pub enum CpuMessage {
    Snapshot(CpuInfo),
}

pub struct CpuPage {
    cpu_usage_history: VecDeque<f32>,
    cpu_info: Option<CpuInfo>,
}

impl CpuPage {
    pub fn new() -> Self {
        Self {
            cpu_usage_history: VecDeque::from(vec![0.0; 30]),
            cpu_info: None,
        }
    }
}

impl super::Page for CpuPage {
    fn update(&mut self, msg: Message) -> cosmic::app::Task<Message> {
        let tasks = Vec::new();
        match msg {
            Message::CpuPage(CpuMessage::Snapshot(snapshot)) => {
                self.cpu_usage_history
                    .push_back(snapshot.global_utilization_percent as f32 / 100.0);
                self.cpu_usage_history.pop_front();
                self.cpu_info = Some(snapshot);
            }
            _ => {}
        }

        cosmic::app::Task::batch(tasks)
    }

    fn view(&self) -> Element<Message> {
        if let Some(cpu_info) = &self.cpu_info {
            let theme = cosmic::theme::active();
            let cosmic = theme.cosmic();
            widget::row()
                .spacing(cosmic.space_xxs())
                .push(
                    widget::canvas(crate::widget::graph::LineGraph {
                        points: self.cpu_usage_history.iter().cloned().collect(),
                    })
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill),
                )
                .push(widget::settings::view_column(vec![
                    widget::settings::section()
                        .title(fl!("processor-info"))
                        .add(widget::settings::item(
                            fl!("model-name"),
                            cpu_info.model_name.clone().apply(widget::text::body),
                        ))
                        .add(widget::settings::item(
                            fl!("physical-cores"),
                            cpu_info
                                .physical_cores
                                .to_string()
                                .apply(widget::text::body),
                        ))
                        .add(widget::settings::item(
                            fl!("logical-cores"),
                            cpu_info
                                .logical_cores
                                .to_string()
                                .clone()
                                .apply(widget::text::body),
                        ))
                        .add(widget::settings::item(
                            fl!("l1-instruction-cache"),
                            cpu_info
                                .cache_info
                                .map(|ci| ci.l1_instruction_kb as u64 * 1024)
                                .unwrap_or_default()
                                .apply(get_bytes)
                                .apply(widget::text::body),
                        ))
                        .add(widget::settings::item(
                            fl!("l1-data-cache"),
                            cpu_info
                                .cache_info
                                .map(|ci| ci.l1_data_kb as u64 * 1024)
                                .unwrap_or_default()
                                .apply(get_bytes)
                                .apply(widget::text::body),
                        ))
                        .add(widget::settings::item(
                            fl!("l2-cache"),
                            cpu_info
                                .cache_info
                                .map(|ci| ci.l2_kb as u64 * 1024)
                                .unwrap_or_default()
                                .apply(get_bytes)
                                .apply(widget::text::body),
                        ))
                        .add(widget::settings::item(
                            fl!("l3-cache"),
                            cpu_info
                                .cache_info
                                .map(|ci| ci.l3_kb as u64 * 1024)
                                .unwrap_or_default()
                                .apply(get_bytes)
                                .apply(widget::text::body),
                        ))
                        .add(widget::settings::item(
                            fl!("architecture"),
                            cpu_info.architecture.clone().apply(widget::text::body),
                        ))
                        .apply(Element::from),
                    widget::settings::section()
                        .title(fl!("processor-stats"))
                        .add(widget::settings::item(
                            fl!("frequency"),
                            cpu_info
                                .core_info
                                .iter()
                                .map(|core| core.frequency_mhz)
                                .max_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap_or_default()
                                .apply(|freq| format!("{} GHz", format_number(freq / 1000.0)))
                                .apply(widget::text::body),
                        ))
                        .add(widget::settings::item(
                            fl!("global-utilization"),
                            cpu_info
                                .global_utilization_percent
                                .apply(|util| format!("{}%", format_number(util)))
                                .to_string()
                                .apply(widget::text::body),
                        ))
                        .apply(widget::scrollable)
                        .apply(Element::from),
                ]))
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

                let mut stream = service.stream_cpu_info(request).await.unwrap().into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(message) = message {
                        sender
                            .send(Message::CpuPage(CpuMessage::Snapshot(message)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
