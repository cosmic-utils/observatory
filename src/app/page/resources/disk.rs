use std::{borrow::Cow, collections::HashMap};

use crate::{fl, helpers::get_bytes};
use cosmic::{app::Task, prelude::*, widget};
use lazy_static::lazy_static;

lazy_static! {
    static ref NOT_LOADED: Cow<'static, str> = fl!("not-loaded").into();
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
    devices: HashMap<String, super::DeviceResource>,
    config: Config,
    max_write: u64,
    max_read: u64,
}

impl DiskPage {
    pub fn new(config: Config) -> Self {
        Self {
            devices: HashMap::new(),
            config,
            max_write: 1,
            max_read: 1,
        }
    }
}

impl Page for DiskPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Snapshot(snapshot) => {
                for disk in snapshot.disks.iter() {
                    if !self.devices.contains_key(&disk.0.model) {
                        let mut device = super::DeviceResource::new();
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
                        self.devices.insert(disk.0.model.clone(), device);
                    }
                    let device = self.devices.get_mut(&disk.0.model).unwrap();
                    device.add_info(DISK_MODEL.clone(), disk.0.model.clone());
                    device.add_info(DISK_DEV.clone(), disk.0.device.clone());
                    device.add_info(DISK_CAP.clone(), get_bytes(disk.0.size));

                    device.set_statistic(DISK_READ.clone(), get_bytes(disk.1.read));
                    device.set_statistic(DISK_WRITE.clone(), get_bytes(disk.1.write));

                    device.push_graph(DISK_READ.clone(), disk.1.read as f32);
                    device.push_graph(DISK_WRITE.clone(), disk.1.write as f32);

                    self.max_read = self.max_read.max(disk.1.read);
                    self.max_write = self.max_write.max(disk.1.write);
                    device.map_graph(DISK_READ.clone(), self.max_read as f32);
                    device.map_graph(DISK_WRITE.clone(), self.max_write as f32);
                }
            }
            Message::UpdateConfig(config) => self.config = config,
            Message::ResourcePage(ResourceMessage::SelectDeviceTab(tab)) => {
                for device in self.devices.iter_mut() {
                    if device.1.contains_tab(tab) {
                        device.1.activate_tab(tab)
                    }
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();
        widget::column::with_children(self.devices.iter().map(|device| device.1.view()).collect())
            .spacing(cosmic.space_xs())
            .apply(Element::from)
    }
}
