use std::borrow::Cow;

use crate::{app::Message, config::Config, fl, helpers::get_bytes};
use cosmic::{app::Task, prelude::*};
use lazy_static::lazy_static;

use super::ResourceMessage;

lazy_static! {
    static ref NOT_LOADED: Cow<'static, str> = fl!("not-loaded").into();
    static ref MEM_USAGE: Cow<'static, str> = fl!("mem-usage").into();
    static ref SWP_USAGE: Cow<'static, str> = fl!("swp-usage").into();
    static ref MEM_CAP: Cow<'static, str> = fl!("mem-cap").into();
    static ref SWP_CAP: Cow<'static, str> = fl!("swp-cap").into();
}

pub struct MemoryPage {
    device: super::DeviceResource,

    config: Config,
}

impl MemoryPage {
    pub fn new(config: Config) -> Self {
        let mut device = super::DeviceResource::new();
        device.add_graph(
            MEM_USAGE.clone().into_owned(),
            crate::widget::graph::LineGraph {
                points: vec![0.0; 30],
            },
        );
        device.activate_graph(0);
        Self { device, config }
    }
}

impl super::super::Page for MemoryPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Snapshot(snapshot) => {
                self.device.add_info(
                    MEM_CAP.clone(),
                    get_bytes(snapshot.mem.0.resident_capacity as u64),
                );
                self.device.add_info(
                    SWP_CAP.clone(),
                    get_bytes(snapshot.mem.0.swap_capacity as u64),
                );
                self.device
                    .set_statistic(MEM_USAGE.clone(), get_bytes(snapshot.mem.1.resident as u64));
                self.device
                    .set_statistic(SWP_USAGE.clone(), get_bytes(snapshot.mem.1.swap as u64));
                self.device.push_graph(
                    MEM_USAGE.clone(),
                    ((snapshot.mem.1.resident + snapshot.mem.1.swap) as f64
                        / (snapshot.mem.0.resident_capacity + snapshot.mem.0.swap_capacity) as f64)
                        as f32,
                );
            }
            Message::UpdateConfig(config) => self.config = config,

            Message::ResourcePage(ResourceMessage::SelectDeviceTab(tab)) => {
                if self.device.contains_tab(tab) {
                    self.device.activate_tab(tab)
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        self.device.view()
    }
}
