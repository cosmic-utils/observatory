use crate::{app::Message, config::Config, fl, helpers::get_bytes};
use cosmic::{app::Task, iced, prelude::*, widget};
use lazy_static::lazy_static;
use monitord::system::{cpu::CpuDynamic, CpuStatic};
use std::collections::VecDeque;

lazy_static! {
    static ref NOT_LOADED: String = fl!("not-loaded");
    // Statistics
    static ref CPU_STATS: String = fl!("cpu-stats");
    static ref CPU_SPEED: String = fl!("cpu-speed");
    static ref CPU_USAGE: String = fl!("cpu-usage");
    // Static info
    static ref CPU_INFO: String = fl!("cpu-info");
    static ref CPU_MODEL: String = fl!("cpu-model");
    static ref CPU_CORES: String = fl!("cpu-cores");
    static ref CPU_PHYS: String = fl!("cpu-physical");
    static ref CPU_LOGI: String = fl!("cpu-logical");

    static ref CPU_CACHE: String = fl!("cpu-cache");
}

pub struct CpuPage {
    cpu_info: Option<CpuStatic>,
    cpu_dyn: Option<CpuDynamic>,

    // Configuration data that persists between application runs.
    config: Config,

    usage_history: VecDeque<f32>,
    per_core_usage_history: Vec<VecDeque<f32>>,
}

impl CpuPage {
    pub fn new(config: Config) -> Self {
        Self {
            cpu_info: None,
            cpu_dyn: None,
            config,
            usage_history: vec![0.0; 30].into(),
            per_core_usage_history: Vec::new(),
        }
    }
}

impl super::super::Page for CpuPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Snapshot(snapshot) => {
                self.cpu_info = Some(snapshot.cpu.0.clone());
                self.cpu_dyn = Some(snapshot.cpu.1.clone());
                self.usage_history.push_back(snapshot.cpu.1.usage);
                self.usage_history.pop_front();

                if self.per_core_usage_history.is_empty() {
                    self.per_core_usage_history
                        .resize(snapshot.cpu.1.usage_by_core.len(), vec![0.0; 30].into())
                }
                for (usage_history, usage) in self
                    .per_core_usage_history
                    .iter_mut()
                    .zip(snapshot.cpu.1.usage_by_core.iter().cloned())
                {
                    usage_history.push_back(usage);
                    usage_history.pop_front();
                }
            }
            Message::UpdateConfig(config) => self.config = config,
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        widget::responsive(|size| {
            let theme = cosmic::theme::active();
            let cosmic = theme.cosmic();
            widget::row()
                .spacing(cosmic.space_xs())
                .push(if self.config.multicore_view {
                    widget::flex_row(
                        self.per_core_usage_history
                            .iter()
                            .map(|usage_history| {
                                widget::canvas(crate::widget::graph::LineGraph {
                                    points: usage_history
                                        .iter()
                                        .cloned()
                                        .map(|value| value / 100.0)
                                        .collect::<Vec<f32>>(),
                                })
                                .apply(Element::from)
                            })
                            .collect::<Vec<Element<Message>>>(),
                    )
                    .justify_content(Some(widget::JustifyContent::SpaceBetween))
                    .align_items(iced::Alignment::Center)
                    .apply(widget::container)
                    .width(iced::Length::Fill)
                } else {
                    widget::canvas(crate::widget::graph::LineGraph {
                        points: self
                            .usage_history
                            .iter()
                            .cloned()
                            .map(|value| value / 100.0)
                            .collect::<Vec<f32>>(),
                    })
                    .width(size.width.min(size.height))
                    .height(size.height.min(size.width))
                    .apply(widget::container)
                    .width(iced::Length::Fill)
                })
                .push(
                    widget::settings::view_column(vec![
                        widget::settings::section()
                            .title(CPU_STATS.as_str())
                            .add(widget::settings::item(
                                CPU_SPEED.as_str(),
                                widget::text::body(
                                    self.cpu_dyn
                                        .as_ref()
                                        .map(|cpudyn| {
                                            format!(
                                                "{} GHz",
                                                crate::helpers::format_number(
                                                    cpudyn.speed as f64 / 1000.0
                                                )
                                            )
                                        })
                                        .unwrap_or(NOT_LOADED.to_string()),
                                ),
                            ))
                            .add(widget::settings::item(
                                CPU_USAGE.as_str(),
                                widget::text::body(
                                    self.cpu_dyn
                                        .as_ref()
                                        .map(|cpudyn| format!("{}%", cpudyn.usage.round()))
                                        .unwrap_or(NOT_LOADED.to_string()),
                                ),
                            ))
                            .apply(Element::from),
                        widget::settings::section()
                            .title(CPU_INFO.as_str())
                            .add(widget::settings::item(
                                CPU_MODEL.as_str(),
                                widget::text::body(
                                    self.cpu_info
                                        .as_ref()
                                        .map(|cpu_inf| cpu_inf.model.clone())
                                        .unwrap_or(NOT_LOADED.to_string()),
                                ),
                            ))
                            .add(widget::settings::item(
                                CPU_CORES.as_str(),
                                widget::text::body(
                                    self.cpu_info
                                        .as_ref()
                                        .map(|cpu_inf| {
                                            format!(
                                                "{} {}, {} {}",
                                                cpu_inf.physical_cores,
                                                CPU_PHYS.as_str(),
                                                cpu_inf.logical_cores,
                                                CPU_LOGI.as_str(),
                                            )
                                        })
                                        .unwrap_or(NOT_LOADED.to_string()),
                                ),
                            ))
                            .add(widget::settings::item(
                                CPU_CACHE.as_str(),
                                widget::column().extend(
                                    self.cpu_info
                                        .as_ref()
                                        .map(|cpu_inf| {
                                            cpu_inf
                                                .caches
                                                .iter()
                                                .map(|cache| {
                                                    widget::text::caption(format!(
                                                        "L{} {}: {}",
                                                        cache.level,
                                                        match cache.cache_type {
                                                                monitord::system::cpu::CacheType::Null => fl!("cpu-cache-null"),
                                                                monitord::system::cpu::CacheType::Data => fl!("cpu-cache-data"),
                                                                monitord::system::cpu::CacheType::Instruction => fl!("cpu-cache-inst"),
                                                                monitord::system::cpu::CacheType::Unified => fl!("cpu-cache-unif"),
                                                                monitord::system::cpu::CacheType::Reserved => fl!("cpu-cache-resv"),
                                                        },
                                                        get_bytes(cache.size as u64)
                                                    ))
                                                    .apply(Element::from)
                                                })
                                                .collect::<Vec<Element<Message>>>()
                                        })
                                        .unwrap_or(vec![]),
                                ),
                            ))
                            .apply(Element::from),
                    ])
                    .apply(widget::container)
                    .width(iced::Length::Fill),
                )
                .apply(Element::from)
        })
        .apply(widget::container)
        .align_x(iced::Alignment::Center)
        .apply(Element::from)
    }
}
