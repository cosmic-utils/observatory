use std::collections::{HashMap, VecDeque};

use cosmic::{
    iced::{self, stream, Subscription},
    prelude::*,
    widget,
};
use futures_util::SinkExt;
use monitord_protocols::{
    monitord::{GpuInfo, GpuList, SnapshotRequest},
    protocols::MonitordServiceClient,
};

use crate::{app::Message, fl};

#[derive(Debug, Clone)]
pub enum GpuMessage {
    Snapshot(GpuList),
    SelectTab(widget::segmented_button::Entity),
}

struct GpuDevice {
    info: GpuInfo,
    history: VecDeque<f32>,
}

pub struct GpuPage {
    gpu_list: widget::segmented_button::SingleSelectModel,
    name_to_entity: HashMap<String, widget::segmented_button::Entity>,
}

impl GpuPage {
    pub fn new() -> Self {
        Self {
            gpu_list: widget::segmented_button::SingleSelectModel::default(),
            name_to_entity: HashMap::new(),
        }
    }
}

impl super::Page for GpuPage {
    fn update(&mut self, msg: Message) -> cosmic::app::Task<Message> {
        let tasks = Vec::new();

        match msg {
            Message::GpuPage(GpuMessage::Snapshot(snapshot)) => {
                for gpu in snapshot.gpus.iter() {
                    let entity = if let Some(entity) = self.name_to_entity.get(&gpu.name) {
                        entity.clone()
                    } else {
                        let entity = self
                            .gpu_list
                            .insert()
                            .text(gpu.name.clone())
                            .data(GpuDevice {
                                info: gpu.clone(),
                                history: VecDeque::from(vec![0.0; 30]),
                            })
                            .id();
                        self.name_to_entity.insert(gpu.name.clone(), entity.clone());
                        entity
                    };
                    let device = self.gpu_list.data_mut::<GpuDevice>(entity).unwrap();
                    device.info = gpu.clone();
                    device
                        .history
                        .push_back(gpu.core_utilization_percent as f32 / 100.0);
                    device.history.pop_front();
                }
            }
            Message::GpuPage(GpuMessage::SelectTab(tab)) => self.gpu_list.activate(tab),
            _ => {}
        }

        cosmic::app::Task::batch(tasks)
    }

    fn view(&self) -> cosmic::Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();

        widget::column()
            .spacing(cosmic.space_xs())
            .push(
                widget::tab_bar::horizontal(&self.gpu_list)
                    .on_activate(|entity| Message::GpuPage(GpuMessage::SelectTab(entity))),
            )
            .push_maybe(self.gpu_list.active_data::<GpuDevice>().map(|gpu| {
                widget::row()
                    .spacing(cosmic.space_xxs())
                    .push(
                        widget::canvas(crate::widget::graph::LineGraph {
                            points: gpu.history.iter().cloned().collect(),
                        })
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill),
                    )
                    .push(
                        widget::settings::view_column(vec![
                            widget::settings::section()
                                .title(fl!("gpu-info"))
                                .add(widget::settings::item(
                                    fl!("gpu-name"),
                                    gpu.info.name.clone().apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("gpu-vendor"),
                                    gpu.info.vendor.clone().apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("vram-total"),
                                    gpu.info
                                        .vram_total_bytes
                                        .apply(crate::helpers::get_bytes)
                                        .apply(widget::text::body),
                                ))
                                .add_maybe(gpu.info.driver_info.as_ref().map(|driv| {
                                    widget::settings::item(
                                        fl!("gpu-kernel-driver"),
                                        driv.kernel_driver.clone().apply(widget::text::body),
                                    )
                                }))
                                .add_maybe(gpu.info.driver_info.as_ref().map(|driv| {
                                    widget::settings::item(
                                        fl!("gpu-user-driver"),
                                        driv.userspace_driver.clone().apply(widget::text::body),
                                    )
                                }))
                                .add_maybe(gpu.info.driver_info.as_ref().map(|driv| {
                                    widget::settings::item(
                                        fl!("gpu-driver-version"),
                                        driv.driver_version.clone().apply(widget::text::body),
                                    )
                                }))
                                .apply(Element::from),
                            widget::settings::section()
                                .title(fl!("gpu-stats"))
                                .add(widget::settings::item(
                                    fl!("vram-used"),
                                    gpu.info
                                        .vram_used_bytes
                                        .apply(crate::helpers::get_bytes)
                                        .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("core-utilization"),
                                    format!(
                                        "{}%",
                                        gpu.info
                                            .core_utilization_percent
                                            .apply(crate::helpers::format_number)
                                    )
                                    .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("vram-utilization"),
                                    format!(
                                        "{}%",
                                        gpu.info
                                            .memory_utilization_percent
                                            .apply(crate::helpers::format_number)
                                    )
                                    .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("gpu-temperature"),
                                    format!(
                                        "{}°C",
                                        gpu.info
                                            .temperature_celsius
                                            .apply(crate::helpers::format_number)
                                    )
                                    .apply(widget::text::body),
                                ))
                                .add_maybe(gpu.info.power_usage_watts.map(|power| {
                                    widget::settings::item(
                                        fl!("gpu-power"),
                                        format!("{} W", power.apply(crate::helpers::format_number))
                                            .apply(widget::text::body),
                                    )
                                }))
                                .add_maybe(gpu.info.core_frequency_mhz.map(|frequency| {
                                    widget::settings::item(
                                        fl!("gpu-core-frequency"),
                                        format!(
                                            "{} MHz",
                                            frequency.apply(crate::helpers::format_number)
                                        )
                                        .apply(widget::text::body),
                                    )
                                }))
                                .add_maybe(gpu.info.memory_frequency_mhz.map(|frequency| {
                                    widget::settings::item(
                                        fl!("gpu-vram-frequency"),
                                        format!(
                                            "{} MHz",
                                            frequency.apply(crate::helpers::format_number)
                                        )
                                        .apply(widget::text::body),
                                    )
                                }))
                                .add_maybe(gpu.info.encoder_info.map(|encoder| {
                                    widget::settings::item(
                                        fl!("gpu-encode"),
                                        format!(
                                            "{}%",
                                            encoder
                                                .video_encode_utilization_percent
                                                .apply(crate::helpers::format_number)
                                        )
                                        .apply(widget::text::body),
                                    )
                                }))
                                .add_maybe(gpu.info.encoder_info.map(|encoder| {
                                    widget::settings::item(
                                        fl!("gpu-decode"),
                                        format!(
                                            "{}%",
                                            encoder
                                                .video_decode_utilization_percent
                                                .apply(crate::helpers::format_number)
                                        )
                                        .apply(widget::text::body),
                                    )
                                }))
                                .add(widget::settings::item(
                                    fl!("gpu-processes"),
                                    gpu.info
                                        .process_info
                                        .len()
                                        .to_string()
                                        .apply(widget::text::body),
                                ))
                                .apply(widget::scrollable)
                                .apply(Element::from),
                        ])
                        .apply(widget::scrollable),
                    )
                    .apply(Element::from)
            }))
            .apply(Element::from)
    }

    fn subscription(&self) -> Vec<Subscription<Message>> {
        vec![Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let mut service = MonitordServiceClient::connect("http://127.0.0.1:50051")
                    .await
                    .unwrap();

                let request = tonic::Request::new(SnapshotRequest { interval_ms: 1000 });

                let mut stream = service.stream_gpu_info(request).await.unwrap().into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(message) = message {
                        sender
                            .send(Message::GpuPage(GpuMessage::Snapshot(message)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
