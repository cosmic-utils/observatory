use crate::app::message::AppMessage;
use crate::fl;

use crate::core::system_info::mem_info::MemInfo;
use cosmic::{app::Task, cosmic_theme, iced, theme, widget, Element};
use std::collections::VecDeque;

pub struct MemResources {
    mem_usage_history: VecDeque<f32>,

    memory_usage: usize,
    total_memory: usize,

    swap_usage: usize,
    total_swap: usize,
}

impl super::Page for MemResources {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::SysInfoRefresh => {
                if let Some(mem_info) = MemInfo::load() {
                    let used = mem_info
                        .mem_total
                        .saturating_sub(mem_info.mem_available + mem_info.dirty)
                        as f64
                        / mem_info.mem_total as f64;

                    self.mem_usage_history.push_back(used as f32);
                    if self.mem_usage_history.len() > 60 {
                        self.mem_usage_history.pop_front();
                    }

                    self.memory_usage = (used * 100.0) as usize;
                    self.total_memory = mem_info.mem_total;

                    self.swap_usage = (mem_info.swap_total.saturating_sub(mem_info.swap_free)
                        as f64
                        / mem_info.swap_total as f64
                        * 100.0) as usize;
                    self.total_swap = mem_info.swap_total;
                }
            }
            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let theme = theme::active();
        let cosmic = theme.cosmic();
        widget::layer_container(
            widget::row()
                .spacing(cosmic.space_xxs())
                .push(self.graph())
                .push(self.info_column(&cosmic)),
        )
        .layer(cosmic_theme::Layer::Background)
        .into()
    }
}

impl MemResources {
    pub fn new() -> Self {
        Self {
            mem_usage_history: VecDeque::from([0.0; 60]),

            memory_usage: 0,
            total_memory: 0,
            swap_usage: 0,
            total_swap: 0,
        }
    }

    fn graph(&self) -> Element<AppMessage> {
        // Usage graph
        widget::layer_container(
            widget::canvas(crate::widgets::line_graph::LineGraph {
                steps: 59,
                points: self.mem_usage_history.clone(),
                autoscale: false,
            })
            .height(iced::Length::Fill)
            .width(iced::Length::Fill),
        )
        .layer(cosmic_theme::Layer::Primary)
        .width(iced::Length::Fill)
        .into()
    }

    fn info_column(&self, cosmic: &theme::CosmicTheme) -> Element<AppMessage> {
        widget::layer_container(
            widget::column()
                .spacing(cosmic.space_s())
                .push(widget::text::title4(fl!("memory")))
                .push(iced::widget::horizontal_rule(1))
                // Total Memory
                .push(
                    widget::row()
                        .align_y(iced::Alignment::Center)
                        .push(widget::text::heading(fl!("capacity")))
                        .push(widget::horizontal_space())
                        .push(widget::text::heading(format!(
                            "{}",
                            format_size(self.total_memory)
                        ))),
                )
                // Utilization
                .push(
                    widget::row()
                        .align_y(iced::Alignment::Center)
                        .push(widget::text::heading(fl!("utilization")))
                        .push(widget::horizontal_space())
                        .push(widget::text::heading(format!("{}%", self.memory_usage))),
                )
                .push(widget::text::title4(fl!("swap")))
                .push(iced::widget::horizontal_rule(1))
                // Total Swap
                .push(
                    widget::row()
                        .align_y(iced::Alignment::Center)
                        .push(widget::text::heading(fl!("capacity")))
                        .push(widget::horizontal_space())
                        .push(widget::text::heading(format!(
                            "{}",
                            format_size(self.total_swap)
                        ))),
                )
                // Utilization
                .push(
                    widget::row()
                        .align_y(iced::Alignment::Center)
                        .push(widget::text::heading(fl!("utilization")))
                        .push(widget::horizontal_space())
                        .push(widget::text::heading(format!("{}%", self.swap_usage))),
                ),
        )
        .layer(cosmic_theme::Layer::Primary)
        .width(iced::Length::Fixed(280.))
        .height(iced::Length::Fill)
        .padding([cosmic.space_s(), cosmic.space_m()])
        .into()
    }
}

fn calc_usage_percentage(used: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64) * 100.0
    }
}

fn format_size(size: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;
    const TB: usize = 1024 * GB;

    if size >= TB {
        format!("{:.1} TiB", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.1} GiB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MiB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KiB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}
