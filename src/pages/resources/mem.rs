use crate::app::message::AppMessage;
use crate::fl;

use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Length;
use cosmic::iced_widget::horizontal_rule;
use cosmic::{theme, widget, Element, Task};
use std::collections::VecDeque;

pub struct MemResources {
    mem_usage_history: VecDeque<f32>,

    memory_usage: usize,
    total_memory: usize,

    swap_usage: usize,
    total_swap: usize,
}

impl super::Page for MemResources {
    fn update(
        &mut self,
        sys: &sysinfo::System,
        message: crate::app::message::AppMessage,
    ) -> cosmic::Task<cosmic::app::message::Message<crate::app::message::AppMessage>> {
        match message {
            AppMessage::Refresh => {
                self.mem_usage_history.push_back(
                    calc_usage_percentage(sys.used_memory() as usize, sys.total_memory() as usize)
                        as f32
                        / 100.,
                );
                if self.mem_usage_history.len() > 60 {
                    self.mem_usage_history.pop_front();
                }

                self.memory_usage = sys.used_memory() as usize;
                self.total_memory = sys.total_memory() as usize;

                self.swap_usage = sys.used_swap() as usize;
                self.total_swap = sys.total_swap() as usize;
            }
            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let theme = theme::active();
        let cosmic = theme.cosmic();
        let mem_name = widget::container(
            widget::text::title3(fl!("memory"))
                .width(Length::Fill)
                .height(Length::Shrink)
                .align_x(Horizontal::Left)
                .align_y(Vertical::Center),
        );

        let page =
            widget::row::with_children::<AppMessage>(vec![self.graph(), self.info_column(&cosmic)])
                .spacing(cosmic.space_s());

        widget::container(
            widget::column::with_children(vec![
                mem_name.into(),
                horizontal_rule(1).into(),
                page.into(),
            ])
            .spacing(cosmic.space_xs()),
        )
        .padding([cosmic.space_xxs(), cosmic.space_xxs()])
        .width(Length::Fill)
        .height(Length::Fill)
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
        widget::container(
            widget::canvas(crate::widgets::line_graph::LineGraph {
                steps: 59,
                points: self.mem_usage_history.clone(),
                autoscale: false,
            })
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }

    fn info_column(&self, cosmic: &theme::CosmicTheme) -> Element<AppMessage> {
        let mut col = widget::column::with_capacity(10);
        col = col.push(
            widget::row::with_children(vec![
                widget::column::with_children(vec![
                    widget::text::heading(fl!("total-mem")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(self.total_memory))).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                widget::column::with_children(vec![
                    widget::text::heading(fl!("total-swap")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(self.total_swap))).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
            ])
            .spacing(cosmic.space_xxs()),
        );

        col = col.push(
            widget::row::with_children(vec![widget::column::with_children(vec![
                widget::text::heading(fl!("mem-utilization")).into(),
                horizontal_rule(1).into(),
                widget::text::body(format!(
                    "{} ({:.1}%)",
                    format_size(self.memory_usage),
                    calc_usage_percentage(self.memory_usage, self.total_memory)
                ))
                .into(),
            ])
            .spacing(cosmic.space_xxxs())
            .into()])
            .spacing(cosmic.space_xxs()),
        );

        col = col.push(
            widget::row::with_children(vec![widget::column::with_children(vec![
                widget::text::heading(fl!("swap-utilization")).into(),
                horizontal_rule(1).into(),
                widget::text::body(format!(
                    "{} ({:.1}%)",
                    format_size(self.swap_usage),
                    calc_usage_percentage(self.swap_usage, self.total_swap)
                ))
                .into(),
            ])
            .spacing(cosmic.space_xxxs())
            .into()])
            .spacing(cosmic.space_xxs()),
        );

        widget::container(
            col.width(Length::Fixed(256.))
                .height(Length::Fill)
                .spacing(cosmic.space_s())
                .padding([cosmic.space_xs(), cosmic.space_xs()]),
        )
        .class(theme::Container::Primary)
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
