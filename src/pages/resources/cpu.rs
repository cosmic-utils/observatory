use crate::app::message::Message;
use crate::fl;

use cosmic::iced::{
    alignment::{Horizontal, Vertical},
    widget::horizontal_rule,
    Length,
};
use cosmic::{theme, widget, Element, Task};
use std::collections::{HashMap, VecDeque};
use sysinfo::System;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextMenuAction {
    MulticoreView(bool),
}

impl widget::menu::Action for ContextMenuAction {
    type Message = Message;
    fn message(&self) -> Self::Message {
        match self {
            ContextMenuAction::MulticoreView(visible) => Message::MulticoreView(visible.clone()),
        }
    }
}

pub struct CpuResources {
    cpu_id: raw_cpuid::CpuId<raw_cpuid::CpuIdReaderNative>,

    cpu_usage_history: VecDeque<f32>,
    core_usage_history: Vec<VecDeque<f32>>,

    cpu_usage: u32,
    cpu_avg: u64,
    process_count: usize,
    thread_count: usize,
    descriptor_count: String, // TODO: Add process file descriptor parsing

    multicore_view: bool,
}

impl super::Page for CpuResources {
    fn update(&mut self, sys: &System, message: Message) -> Task<cosmic::app::Message<Message>> {
        match message {
            Message::Refresh => {
                self.update_usage_graphs(&sys);
                self.update_metrics(&sys);
            }
            Message::MulticoreView(checked) => {
                self.multicore_view = checked;
            }
            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = theme::active();
        let cosmic = theme.cosmic();
        let cpu_name = widget::container(
            widget::text::title3(format!(
                "Processor  —  {}",
                self.cpu_id
                    .get_processor_brand_string()
                    .unwrap()
                    .as_str()
                    .to_string()
            ))
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center),
        );
        let page =
            widget::row::with_children::<Message>(vec![self.graph(), self.info_column(&cosmic)])
                .spacing(cosmic.space_s());

        widget::container(
            widget::column::with_children(vec![
                cpu_name.into(),
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

impl CpuResources {
    pub fn new() -> Self {
        Self {
            cpu_id: raw_cpuid::CpuId::new(),
            cpu_usage_history: VecDeque::from([0.0; 60]),
            core_usage_history: Vec::new(),

            cpu_usage: 0,
            cpu_avg: 0,
            process_count: 0,
            thread_count: 0,
            descriptor_count: String::from("TODO"),
            multicore_view: false,
        }
    }

    fn update_usage_graphs(&mut self, sys: &System) {
        // Cpu Usage History Graph
        self.cpu_usage_history
            .push_back(sys.global_cpu_usage() / 100.);
        if self.cpu_usage_history.len() > 60 {
            self.cpu_usage_history.pop_front();
        }

        // Core usage history
        let cpus = sys.cpus();
        if self.core_usage_history.len() == 0 {
            for _ in 0..cpus.len() {
                self.core_usage_history.push(VecDeque::from([0.0; 60]));
            }
        }
        for i in 0..cpus.len() {
            self.core_usage_history[i].push_back(cpus[i].cpu_usage() / 100.);
            if self.core_usage_history[i].len() > 60 {
                self.core_usage_history[i].pop_front();
            }
        }
    }

    fn update_metrics(&mut self, sys: &System) {
        self.cpu_usage = sys.global_cpu_usage() as u32;
        self.cpu_avg = get_cpu_avg(sys);
        self.process_count = sys
            .processes()
            .iter()
            .filter(|proc| proc.1.thread_kind().is_none())
            .count();
        self.thread_count = sys.processes().len();
        self.descriptor_count = String::from("TODO");
    }

    fn context_menu(&self) -> Option<Vec<widget::menu::Tree<Message>>> {
        Some(widget::menu::items(
            &HashMap::new(),
            vec![
                widget::menu::Item::Button(
                    fl!("multi-core-view"),
                    None,
                    ContextMenuAction::MulticoreView(true),
                ),
                widget::menu::Item::Button(
                    fl!("single-core-view"),
                    None,
                    ContextMenuAction::MulticoreView(false),
                ),
            ],
        ))
    }

    fn graph(&self) -> Element<Message> {
        // Usage graph
        let element = if self.multicore_view {
            let mut grid = widget::column().width(Length::Fill);
            let mut row = widget::row().width(Length::Fill).spacing(10.0);

            for (i, usage) in self.core_usage_history.iter().enumerate() {
                row = row.push(
                    widget::container(
                        widget::column()
                            .push(widget::text::text(format!("{} {}", fl!("core"), i)))
                            .push(
                                widget::canvas(crate::widgets::LineGraph {
                                    steps: 59,
                                    points: usage.clone(),
                                    autoscale: false,
                                })
                                .width(Length::Fill),
                            ),
                    )
                    .width(Length::FillPortion(1)),
                );

                if (i + 1) % 4 == 0 {
                    grid = grid.push(row);
                    grid = grid.push(widget::Space::with_height(5));
                    row = widget::row().width(Length::Fill).spacing(10.0);
                }
            }

            grid = grid.push(row);

            widget::container(widget::scrollable(grid)).width(Length::Fill)
        } else {
            widget::container(
                widget::canvas(crate::widgets::LineGraph {
                    steps: 59,
                    points: self.cpu_usage_history.clone(),
                    autoscale: false,
                })
                .height(Length::Fill)
                .width(Length::Fill),
            )
            .width(Length::Fill)
        };

        let widget = cosmic::widget::context_menu(element, self.context_menu());
        widget.into()
    }

    fn info_column(&self, cosmic: &theme::CosmicTheme) -> Element<Message> {
        let mut col = widget::column::with_capacity(10);
        col = col.push(
            widget::row::with_children(vec![
                // CPU Utilization
                widget::column::with_children(vec![
                    widget::text::heading(fl!("utilization")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}%", self.cpu_usage)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                // CPU Core Speed Average
                widget::column::with_children(vec![
                    widget::text::heading(fl!("speed-avg")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(get_hz(self.cpu_avg)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
            ])
            .spacing(cosmic.space_xxs()),
        );

        col = col.push(
            widget::row::with_children(vec![
                // Process count
                widget::column::with_children(vec![
                    widget::text::heading("Processes").into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", self.process_count)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                // Thread count
                widget::column::with_children(vec![
                    widget::text::heading("Threads").into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", self.thread_count)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                // Handles count
                widget::column::with_children(vec![
                    widget::text::heading("Descriptors").into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", self.descriptor_count)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
            ])
            .spacing(cosmic.space_xxs()),
        );

        let uptime = sysinfo::System::uptime();
        let seconds = uptime % 60;
        let minutes = (uptime / 60) % 60;
        let hours = (uptime / 60) / 60;
        col = col.push(
            widget::column::with_children(vec![
                widget::text::heading("Up time").into(),
                horizontal_rule(1).into(),
                widget::text::body(format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)).into(),
            ])
            .spacing(cosmic.space_xxxs()),
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

fn get_cpu_avg(sys: &sysinfo::System) -> u64 {
    let mut avg = 0;
    for cpu in sys.cpus() {
        avg += cpu.frequency()
    }
    avg / sys.cpus().len() as u64
}

fn get_hz(hz: u64) -> String {
    if hz < 1000u64.pow(1) {
        format!("{}MHz", hz as f64 / 1000f64.powf(0.))
    } else if hz < 1000u64.pow(2) {
        format!("{:.2}GHz", hz as f64 / 1000f64.powf(1.))
    } else {
        format!("{:.2}THz", hz as f64 / 1000f64.powf(2.))
    }
}
