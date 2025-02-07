use crate::{app::Message, config::Config, fl, helpers::get_bytes};
use cosmic::{app::Task, prelude::*};
use lazy_static::lazy_static;
use std::borrow::Cow;

use super::ResourceMessage;

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
        let mut device = super::DeviceResource::new();
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
            Message::Snapshot(snapshot) => {
                self.device
                    .add_info(CPU_MODEL.clone(), snapshot.cpu.0.model.clone());
                self.device.add_info(
                    CPU_CORES.clone(),
                    format!(
                        "{} {} {} {}",
                        snapshot.cpu.0.physical_cores,
                        CPU_PHYS.clone(),
                        snapshot.cpu.0.logical_cores,
                        CPU_LOGI.clone()
                    ),
                );
                self.device
                    .add_info(CPU_CACHE.clone(), format_cache(&snapshot.cpu.0.caches));

                self.device.set_statistic(
                    CPU_USAGE.clone(),
                    format!("{}%", snapshot.cpu.1.usage.round()),
                );
                self.device.set_statistic(
                    CPU_SPEED.clone(),
                    format!(
                        "{} GHz",
                        crate::helpers::format_number(snapshot.cpu.1.speed as f64 / 1000.0)
                    ),
                );
                self.device
                    .push_graph(CPU_USAGE.clone(), snapshot.cpu.1.usage / 100.0);
            }
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

fn format_cache(caches: &Vec<monitord::system::cpu::CpuCache>) -> Cow<'static, str> {
    caches
        .iter()
        .map(|cache| {
            format!(
                "L{} {:?}: {}",
                cache.level,
                cache.cache_type,
                get_bytes(cache.size as u64)
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
        .into()
}
