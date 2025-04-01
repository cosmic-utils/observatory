use std::collections::{HashMap, VecDeque};

use cosmic::{
    iced::{self, stream, Subscription},
    prelude::*,
    widget,
};
use futures_util::SinkExt;
use monitord_protocols::{
    monitord::{SnapshotRequest, StorageInfo, StorageList},
    protocols::MonitordServiceClient,
};

use crate::{app::Message, fl};

#[derive(Debug, Clone)]
pub enum StorageMessage {
    Snapshot(StorageList),
    SelectTab(widget::segmented_button::Entity),
}

struct StorageDevice {
    info: StorageInfo,
    history: VecDeque<f32>,
}

pub struct StoragePage {
    storage_list: widget::segmented_button::SingleSelectModel,
    name_to_entity: HashMap<String, widget::segmented_button::Entity>,
}

impl StoragePage {
    pub fn new() -> Self {
        Self {
            storage_list: widget::segmented_button::SingleSelectModel::default(),
            name_to_entity: HashMap::new(),
        }
    }
}

impl super::Page for StoragePage {
    fn update(&mut self, msg: Message) -> cosmic::app::Task<Message> {
        let tasks = Vec::new();

        match msg {
            Message::StoragePage(StorageMessage::Snapshot(snapshot)) => {
                for storage in snapshot.storages.iter() {
                    let entity = if let Some(entity) = self.name_to_entity.get(&storage.device_name)
                    {
                        entity.clone()
                    } else {
                        let entity = self
                            .storage_list
                            .insert()
                            .text(storage.device_name.clone())
                            .data(StorageDevice {
                                info: storage.clone(),
                                history: VecDeque::from(vec![0.0; 30]),
                            })
                            .id();
                        self.name_to_entity
                            .insert(storage.device_name.clone(), entity.clone());
                        entity
                    };
                    let device = self.storage_list.data_mut::<StorageDevice>(entity).unwrap();
                    device.info = storage.clone();
                    device.history.push_back(
                        storage.write_bytes_per_sec as f32 + storage.read_bytes_per_sec as f32,
                    );
                    device.history.pop_front();
                }
            }
            Message::StoragePage(StorageMessage::SelectTab(tab)) => self.storage_list.activate(tab),
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
                widget::tab_bar::horizontal(&self.storage_list)
                    .on_activate(|entity| Message::StoragePage(StorageMessage::SelectTab(entity))),
            )
            .push_maybe(
                self.storage_list
                    .active_data::<StorageDevice>()
                    .map(|storage| {
                        widget::row()
                            .spacing(cosmic.space_xxs())
                            .push(
                                widget::canvas(crate::widget::graph::LineGraph {
                                    points: {
                                        let max = storage
                                            .history
                                            .iter()
                                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                                            .unwrap()
                                            .max(1.0);
                                        tracing::info!("{}", max);
                                        storage
                                            .history
                                            .iter()
                                            .cloned()
                                            .map(|val| val / max)
                                            .collect()
                                    },
                                })
                                .width(iced::Length::Fill)
                                .height(iced::Length::Fill),
                            )
                            .push(
                                widget::settings::view_column(vec![
                                    widget::settings::section()
                                        .title(fl!("storage-info"))
                                        .add(widget::settings::item(
                                            fl!("storage-device"),
                                            storage
                                                .info
                                                .device_name
                                                .clone()
                                                .apply(widget::text::body),
                                        ))
                                        .add(widget::settings::item(
                                            fl!("storage-type"),
                                            storage
                                                .info
                                                .device_type
                                                .clone()
                                                .apply(widget::text::body),
                                        ))
                                        .add(widget::settings::item(
                                            fl!("model"),
                                            storage.info.model.clone().apply(widget::text::body),
                                        ))
                                        .add_maybe(storage.info.serial_number.as_ref().map(
                                            |serial| {
                                                widget::settings::item(
                                                    fl!("disk-serial"),
                                                    serial.clone().apply(widget::text::body),
                                                )
                                            },
                                        ))
                                        .add_maybe(storage.info.partition_label.as_ref().map(
                                            |plabel| {
                                                widget::settings::item(
                                                    fl!("part-label"),
                                                    plabel.clone().apply(widget::text::body),
                                                )
                                            },
                                        ))
                                        .add(widget::settings::item(
                                            fl!("fs-type"),
                                            storage
                                                .info
                                                .filesystem_type
                                                .clone()
                                                .apply(widget::text::body),
                                        ))
                                        .add(widget::settings::item(
                                            fl!("mount-point"),
                                            storage
                                                .info
                                                .mount_point
                                                .clone()
                                                .apply(widget::text::body),
                                        ))
                                        .add(widget::settings::item(
                                            fl!("disk-space"),
                                            storage
                                                .info
                                                .total_space_bytes
                                                .apply(crate::helpers::get_bytes)
                                                .apply(widget::text::body),
                                        ))
                                        .apply(Element::from),
                                    widget::settings::section()
                                        .title(fl!("storage-stats"))
                                        .add(widget::settings::item(
                                            fl!("available-space"),
                                            storage
                                                .info
                                                .available_space_bytes
                                                .apply(crate::helpers::get_bytes)
                                                .apply(widget::text::body),
                                        ))
                                        .add(widget::settings::item(
                                            fl!("disk-read"),
                                            format!(
                                                "{}/s",
                                                storage
                                                    .info
                                                    .read_bytes_per_sec
                                                    .apply(crate::helpers::get_bytes)
                                            )
                                            .apply(widget::text::body),
                                        ))
                                        .add(widget::settings::item(
                                            fl!("disk-write"),
                                            format!(
                                                "{}/s",
                                                storage
                                                    .info
                                                    .write_bytes_per_sec
                                                    .apply(crate::helpers::get_bytes)
                                            )
                                            .apply(widget::text::body),
                                        ))
                                        .add(widget::settings::item(
                                            fl!("io-time"),
                                            format!("{} ms", storage.info.io_time_ms)
                                                .apply(widget::text::body),
                                        ))
                                        .add_maybe(storage.info.temperature_celsius.map(|temp| {
                                            widget::settings::item(
                                                fl!("disk-temp"),
                                                format!(
                                                    "{}°C",
                                                    temp.apply(crate::helpers::format_number)
                                                )
                                                .apply(widget::text::body),
                                            )
                                        }))
                                        .add_maybe(storage.info.smart_data.as_ref().map(|smart| {
                                            widget::settings::item(
                                                fl!("smart-status"),
                                                smart
                                                    .health_status
                                                    .clone()
                                                    .apply(widget::text::body),
                                            )
                                        }))
                                        .apply(Element::from),
                                ])
                                .apply(widget::scrollable),
                            )
                            .apply(Element::from)
                    }),
            )
            .apply(Element::from)
    }

    fn subscription(&self) -> Vec<Subscription<Message>> {
        vec![Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let mut service = MonitordServiceClient::connect("http://127.0.0.1:50051")
                    .await
                    .unwrap();

                let request = tonic::Request::new(SnapshotRequest { interval_ms: 1000 });

                let mut stream = service
                    .stream_storage_info(request)
                    .await
                    .unwrap()
                    .into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(message) = message {
                        sender
                            .send(Message::StoragePage(StorageMessage::Snapshot(message)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
