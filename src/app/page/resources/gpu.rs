use std::borrow::Cow;

use super::ResourceMessage;
use crate::{app::Message, config::Config, fl, helpers::get_bytes};
use cosmic::{
    app::Task,
    iced::{stream, Subscription},
    prelude::*,
    widget,
};
use futures_util::SinkExt;
use lazy_static::lazy_static;

lazy_static! {
    static ref NOT_SUPPORTED: Cow<'static, str> = fl!("not-supported").into();
    static ref GPU: Cow<'static, str> = fl!("gpu").into();
    static ref GPU_MODEL: Cow<'static, str> = fl!("gpu-model").into();
    static ref GPU_DRIVER: Cow<'static, str> = fl!("gpu-driver").into();
    static ref GPU_VRAM: Cow<'static, str> = fl!("gpu-vram").into();
    static ref GPU_USAGE: Cow<'static, str> = fl!("gpu-usage").into();
    static ref GPU_ENCODE: Cow<'static, str> = fl!("gpu-encode").into();
    static ref GPU_DECODE: Cow<'static, str> = fl!("gpu-decode").into();
    static ref GPU_VRAM_USAGE: Cow<'static, str> = fl!("gpu-vram-usage").into();
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
            Message::ResourcePage(ResourceMessage::GpuSnapshot(snapshot)) => {
                if self.devices.is_empty() {
                    self.devices
                        .extend(snapshot.gpus.iter().enumerate().map(|(index, gpu)| {
                            let mut device =
                                super::DeviceResource::new(format!("{} {}", GPU.clone(), index));
                            device.add_graph(
                                GPU_USAGE.clone(),
                                crate::widget::graph::LineGraph {
                                    points: vec![0.0; 30],
                                },
                            );
                            device.add_graph(
                                GPU_ENCODE.clone(),
                                crate::widget::graph::LineGraph {
                                    points: vec![0.0; 30],
                                },
                            );
                            device.add_graph(
                                GPU_DECODE.clone(),
                                crate::widget::graph::LineGraph {
                                    points: vec![0.0; 30],
                                },
                            );
                            device.activate_graph(0);
                            device.add_info(GPU_MODEL.clone(), gpu.name.clone());
                            device.add_info(
                                GPU_DRIVER.clone(),
                                gpu.driver_info
                                    .clone()
                                    .map(|driv| driv.kernel_driver)
                                    .unwrap_or_default(),
                            );
                            device.add_info(GPU_VRAM.clone(), get_bytes(gpu.vram_total_bytes));

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
                    device.set_statistic(
                        GPU_USAGE.clone(),
                        format!("{}%", gpu.core_utilization_percent.round()),
                    );
                    device.set_statistic(
                        GPU_ENCODE.clone(),
                        if let Some(encoder_info) = gpu.encoder_info {
                            format!("{}%", encoder_info.video_encode_utilization_percent).into()
                        } else {
                            NOT_SUPPORTED.clone()
                        },
                    );
                    device.set_statistic(
                        GPU_DECODE.clone(),
                        if let Some(encoder_info) = gpu.encoder_info {
                            format!("{}%", encoder_info.video_decode_utilization_percent).into()
                        } else {
                            NOT_SUPPORTED.clone()
                        },
                    );
                    device.set_statistic(GPU_VRAM_USAGE.clone(), get_bytes(gpu.vram_used_bytes));
                    device.push_graph(
                        GPU_USAGE.clone(),
                        gpu.core_utilization_percent as f32 / 100.0,
                    );
                    device.push_graph(
                        GPU_ENCODE.clone(),
                        gpu.encoder_info
                            .map(|enc| enc.video_encode_utilization_percent as f32)
                            .unwrap_or_default()
                            / 100.0,
                    );
                    device.push_graph(
                        GPU_DECODE.clone(),
                        gpu.encoder_info
                            .map(|enc| enc.video_encode_utilization_percent as f32)
                            .unwrap_or_default()
                            / 100.0,
                    );
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

                let mut stream = client.stream_gpu_info(request).await.unwrap().into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(item) = message {
                        sender
                            .send(Message::ResourcePage(ResourceMessage::GpuSnapshot(item)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
