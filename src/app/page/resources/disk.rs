use std::borrow::Cow;

use crate::{fl, helpers::get_bytes};
use cosmic::{
    app::Task,
    iced::{stream, Subscription},
    prelude::*,
    widget,
};
use futures_util::SinkExt;
use lazy_static::lazy_static;

lazy_static! {
    static ref NOT_LOADED: Cow<'static, str> = fl!("not-loaded").into();
    static ref DISK: Cow<'static, str> = fl!("disk").into();
    static ref DISK_STATS: Cow<'static, str> = fl!("disk-stats").into();
    static ref DISK_READ: Cow<'static, str> = fl!("disk-read").into();
    static ref DISK_WRITE: Cow<'static, str> = fl!("disk-write").into();
    static ref DISK_MODEL: Cow<'static, str> = fl!("disk-model").into();
    static ref DISK_DEV: Cow<'static, str> = fl!("disk-dev").into();
    static ref DISK_CAP: Cow<'static, str> = fl!("disk-cap").into();
}

use crate::{
    app::{page::Page, Message},
    config::Config,
};

use super::ResourceMessage;

pub struct DiskPage {
    devices: Vec<super::DeviceResource>,
    config: Config,
    active: usize,
    max_write: u64,
    max_read: u64,
}

impl DiskPage {
    pub fn new(config: Config) -> Self {
        Self {
            devices: Vec::new(),
            config,
            active: 0,
            max_write: 1,
            max_read: 1,
        }
    }
}

impl Page for DiskPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::ResourcePage(ResourceMessage::DiskSnapshot(snapshot)) => {
                for (index, disk) in snapshot.storages.iter().enumerate() {
                    if self
                        .devices
                        .iter()
                        .find(|device| {
                            device
                                .get_info(DISK_MODEL.clone())
                                .is_some_and(|model| model == disk.model)
                        })
                        .is_none()
                    {
                        tracing::info!("Pushing disk");
                        let mut device = super::DeviceResource::new(format!("Disk {}", index));
                        device.add_graph(
                            DISK_READ.clone(),
                            crate::widget::graph::LineGraph {
                                points: vec![0.0; 30],
                            },
                        );
                        device.add_graph(
                            DISK_WRITE.clone(),
                            crate::widget::graph::LineGraph {
                                points: vec![0.0; 30],
                            },
                        );
                        device.activate_graph(0);

                        device.add_info(DISK_MODEL.clone(), disk.model.clone());
                        device.add_info(DISK_DEV.clone(), disk.device_name.clone());
                        device.add_info(DISK_CAP.clone(), get_bytes(disk.total_space_bytes));

                        device.apply_mut(|device| {
                            if index != 0 {
                                device.on_prev(Message::ResourcePage(ResourceMessage::DiskPrev));
                            }
                        });
                        device.apply_mut(|device| {
                            if index != snapshot.storages.len() - 1 {
                                device.on_next(Message::ResourcePage(ResourceMessage::DiskNext));
                            }
                        });
                        self.devices.push(device);
                    }
                    let device = self
                        .devices
                        .iter_mut()
                        .find(|device| {
                            device
                                .get_info(DISK_MODEL.clone())
                                .is_some_and(|model| model == disk.model)
                        })
                        .expect("Disk not found!");
                    device.set_statistic(DISK_READ.clone(), get_bytes(disk.read_bytes_per_sec));
                    device.set_statistic(DISK_WRITE.clone(), get_bytes(disk.write_bytes_per_sec));

                    device.push_graph(DISK_READ.clone(), disk.read_bytes_per_sec as f32);
                    device.push_graph(DISK_WRITE.clone(), disk.write_bytes_per_sec as f32);

                    self.max_read = self.max_read.max(disk.read_bytes_per_sec);
                    self.max_write = self.max_write.max(disk.write_bytes_per_sec);
                    device.map_graph(DISK_READ.clone(), self.max_read as f32);
                    device.map_graph(DISK_WRITE.clone(), self.max_write as f32);
                }
            }
            Message::UpdateConfig(config) => self.config = config,
            Message::ResourcePage(ResourceMessage::SelectDeviceTab(tab)) => {
                for device in self.devices.iter_mut() {
                    if device.contains_tab(tab) {
                        device.activate_tab(tab)
                    }
                }
            }

            Message::ResourcePage(ResourceMessage::DiskNext) => self.active += 1,
            Message::ResourcePage(ResourceMessage::DiskPrev) => self.active -= 1,
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        self.devices
            .get(self.active)
            .map(|device| device.view())
            .unwrap_or(widget::horizontal_space().apply(Element::from))
    }

    fn subscription(&self) -> Vec<Subscription<Message>> {
        vec![Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                use monitord_protocols::protocols::MonitordServiceClient;
                let mut client = MonitordServiceClient::connect("http://127.0.0.1:50051")
                    .await
                    .unwrap();

                let request = tonic::Request::new(monitord_protocols::monitord::SnapshotRequest {
                    interval_ms: 1000,
                });

                let mut stream = client
                    .stream_storage_info(request)
                    .await
                    .unwrap()
                    .into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(item) = message {
                        sender
                            .send(Message::ResourcePage(ResourceMessage::DiskSnapshot(item)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
