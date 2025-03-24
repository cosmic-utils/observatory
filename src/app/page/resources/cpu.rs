use crate::{app::Message, config::Config, fl, helpers::get_bytes};
use cosmic::{
    app::Task,
    iced::{stream, Subscription},
    prelude::*,
};
use futures_util::SinkExt;
use lazy_static::lazy_static;
use std::borrow::Cow;

use super::ResourceMessage;
use monitord_protocols::monitord::*;

lazy_static! {
    static ref NOT_LOADED: Cow<'static, str> = fl!("not-loaded").into();
    // Statistics
    static ref CPU_SPEED: Cow<'static, str> = fl!("cpu-speed").into();
    static ref CPU_USAGE: Cow<'static, str> = fl!("cpu-usage").into();
    // Static info
    static ref CPU_MODEL: Cow<'static, str> = fl!("cpu-model").into();
    static ref CPU_CORES: Cow<'static, str> = fl!("cpu-cores").into();
    static ref CPU_PHYS: Cow<'static, str> = fl!("cpu-physical").into();
    static ref CPU_LOGI: Cow<'static, str> = fl!("cpu-logical").into();

    static ref CPU_CACHE: Cow<'static, str> = fl!("cpu-cache").into();
}

pub struct CpuPage {
    device: super::DeviceResource,

    // Configuration data that persists between application runs.
    config: Config,
}

impl CpuPage {
    pub fn new(config: Config) -> Self {
        let mut device = super::DeviceResource::new("");
        device.add_graph(
            CPU_USAGE.clone(),
            crate::widget::graph::LineGraph {
                points: vec![0.0; 30],
            },
        );
        device.activate_graph(0);
        Self { device, config }
    }
}

impl super::super::Page for CpuPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::UpdateConfig(config) => self.config = config,
            Message::ResourcePage(ResourceMessage::SelectDeviceTab(tab)) => {
                if self.device.contains_tab(tab) {
                    self.device.activate_tab(tab)
                }
            }
            Message::ResourcePage(ResourceMessage::CpuSnapshot(snapshot)) => {
                self.device
                    .add_info(CPU_MODEL.clone(), snapshot.model_name.clone());
                self.device.add_info(
                    CPU_CORES.clone(),
                    format!(
                        "{} {} {} {}",
                        snapshot.physical_cores,
                        CPU_PHYS.clone(),
                        snapshot.logical_cores,
                        CPU_LOGI.clone()
                    ),
                );

                self.device.add_info(
                    CPU_CACHE.clone(),
                    format_cache(&snapshot.cache_info.unwrap()),
                );

                self.device.set_statistic(
                    CPU_USAGE.clone(),
                    format!("{}%", snapshot.global_utilization_percent.round()),
                );
                self.device.set_statistic(
                    CPU_SPEED.clone(),
                    format!(
                        "{} GHz",
                        crate::helpers::format_number(
                            snapshot
                                .core_info
                                .iter()
                                .map(|core| core.frequency_mhz as u32)
                                .max()
                                .unwrap_or_default() as f64
                                / 1000.0
                        )
                    ),
                );
                self.device.push_graph(
                    CPU_USAGE.clone(),
                    snapshot.global_utilization_percent as f32 / 100.0,
                );
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        self.device.view()
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

                let mut stream = client.stream_cpu_info(request).await.unwrap().into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(item) = message {
                        sender
                            .send(Message::ResourcePage(ResourceMessage::CpuSnapshot(item)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}

fn format_cache(caches: &CpuCache) -> Cow<'static, str> {
    format!(
        "L1 Instruction: {}\nL1 Data: {}\nL2: {}\nL3: {}",
        get_bytes(caches.l1_instruction_kb as u64 * 1024),
        get_bytes(caches.l1_data_kb as u64 * 1024),
        get_bytes(caches.l2_kb as u64 * 1024),
        get_bytes(caches.l3_kb as u64 * 1024)
    )
    .into()
}
