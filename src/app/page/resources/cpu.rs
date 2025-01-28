use std::collections::VecDeque;

use cosmic::{app::Task, iced, prelude::*, widget};
use monitord::system::{cpu::CpuDynamic, CpuStatic};

use crate::{app::Message, helpers::get_bytes};

pub struct CpuPage {
    cpu_info: Option<CpuStatic>,
    cpu_dyn: Option<CpuDynamic>,

    usage_history: VecDeque<f32>,
}

impl CpuPage {
    pub fn new() -> Self {
        Self {
            cpu_info: None,
            cpu_dyn: None,
            usage_history: vec![0.0; 30].into(),
        }
    }
}

impl super::super::Page for CpuPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Snapshot(snapshot) => {
                self.cpu_info = Some(snapshot.cpu_static_info.clone());
                self.cpu_dyn = Some(snapshot.cpu_dynamic_info.clone());
                self.usage_history
                    .push_back(snapshot.cpu_dynamic_info.usage);
                self.usage_history.pop_front();
            }
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
                .push(
                    widget::canvas(crate::widget::graph::LineGraph {
                        points: self
                            .usage_history
                            .iter()
                            .cloned()
                            .map(|value| value / 100.0)
                            .collect::<Vec<f32>>(),
                    })
                    .width(size.width.min(size.height * 1.2))
                    .height(size.height.min(size.width * 1.2))
                    .apply(widget::container)
                    .width(iced::Length::Fill),
                )
                .push(
                    widget::settings::view_column(vec![
                        widget::settings::section()
                            .title("Statistics")
                            .add(widget::settings::item(
                                "Speed",
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
                                        .unwrap_or("Not Loaded".to_string()),
                                ),
                            ))
                            .add(widget::settings::item(
                                "Usage",
                                widget::text::body(
                                    self.cpu_dyn
                                        .as_ref()
                                        .map(|cpudyn| format!("{}%", cpudyn.usage.round()))
                                        .unwrap_or("Not Loaded".to_string()),
                                ),
                            ))
                            .apply(Element::from),
                        widget::settings::section()
                            .title("CPU Information")
                            .add(widget::settings::item(
                                "Model",
                                widget::text::body(
                                    self.cpu_info
                                        .as_ref()
                                        .map(|cpu_inf| cpu_inf.model.clone())
                                        .unwrap_or("Not Loaded".to_string()),
                                ),
                            ))
                            .add(widget::settings::item(
                                "Cores",
                                widget::text::body(
                                    self.cpu_info
                                        .as_ref()
                                        .map(|cpu_inf| {
                                            format!(
                                                "{} Physical, {} Logical",
                                                cpu_inf.physical_cores, cpu_inf.logical_cores
                                            )
                                        })
                                        .unwrap_or("Not Loaded".to_string()),
                                ),
                            ))
                            .add(widget::settings::item(
                                "Cache",
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
                                                        cache.cache_type,
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
