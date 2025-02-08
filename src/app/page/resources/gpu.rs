use std::borrow::Cow;

use super::ResourceMessage;
use crate::{app::Message, config::Config, fl, helpers::get_bytes};
use cosmic::{app::Task, prelude::*, widget};
use lazy_static::lazy_static;

lazy_static! {
    static ref GPU: Cow<'static, str> = fl!("gpu").into();
}

pub struct GpuPage {
    devices: Vec<super::DeviceResource>,
    active: usize,
    config: Config,
}

impl GpuPage {
    pub fn new(config: Config) -> Self {
        Self {
            devices: Vec::new(),
            active: 0,
            config,
        }
    }
}

impl super::super::Page for GpuPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::UpdateConfig(config) => self.config = config,
            Message::Snapshot(snapshot) => {
                if self.devices.is_empty() {
                    self.devices
                        .extend(snapshot.gpus.iter().enumerate().map(|(index, gpu)| {
                            let mut device =
                                super::DeviceResource::new(format!("{} {}", GPU.clone(), index));
                            device.add_graph(
                                "GPU Usage",
                                crate::widget::graph::LineGraph {
                                    points: vec![0.0; 30],
                                },
                            );
                            device.add_graph(
                                "Encode",
                                crate::widget::graph::LineGraph {
                                    points: vec![0.0; 30],
                                },
                            );
                            device.add_graph(
                                "Decode",
                                crate::widget::graph::LineGraph {
                                    points: vec![0.0; 30],
                                },
                            );
                            device.activate_graph(0);
                            device.add_info("Model", gpu.0.name.clone());
                            device.add_info("Driver Version", gpu.0.driver.clone());
                            device.add_info("Video Memory", get_bytes(gpu.0.video_memory));

                            device.apply_mut(|device| {
                                if index != 0 {
                                    device.on_prev(Message::ResourcePage(ResourceMessage::GpuPrev));
                                }
                            });
                            device.apply_mut(|device| {
                                if index != snapshot.gpus.len() - 1 {
                                    device.on_next(Message::ResourcePage(ResourceMessage::GpuNext));
                                }
                            });
                            device
                        }));
                }
                for (gpu, device) in snapshot.gpus.iter().zip(self.devices.iter_mut()) {
                    device.set_statistic("Usage", format!("{}%", gpu.1.usage.round()));
                    device.set_statistic("Encode", format!("{}%", gpu.1.enc.round()));
                    device.set_statistic("Decode", format!("{}%", gpu.1.dec.round()));
                    device.set_statistic("Memory Usage", get_bytes(gpu.1.video_mem));
                    device.push_graph("GPU Usage", gpu.1.usage / 100.0);
                    device.push_graph("Encode", gpu.1.enc / 100.0);
                    device.push_graph("Decode", gpu.1.dec / 100.0);
                }
            }
            Message::ResourcePage(ResourceMessage::SelectDeviceTab(tab)) => {
                for device in self.devices.iter_mut() {
                    if device.contains_tab(tab) {
                        device.activate_tab(tab);
                    }
                }
            }
            Message::ResourcePage(ResourceMessage::GpuNext) => self.active += 1,
            Message::ResourcePage(ResourceMessage::GpuPrev) => self.active -= 1,

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
}
