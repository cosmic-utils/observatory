use super::ResourceMessage;
use crate::{app::Message, config::Config, helpers::get_bytes};
use cosmic::{app::Task, prelude::*, widget};

pub struct GpuPage {
    devices: Vec<super::DeviceResource>,

    config: Config,
}

impl GpuPage {
    pub fn new(config: Config) -> Self {
        Self {
            devices: Vec::new(),
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
                    self.devices.extend(snapshot.gpus.iter().map(|gpu| {
                        let mut device = super::DeviceResource::new();
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

            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();
        widget::column()
            .extend(self.devices.iter().map(|device| device.view()))
            .spacing(cosmic.space_s())
            .apply(widget::scrollable)
            .apply(Element::from)
    }
}
