use crate::app::message::AppMessage;
use crate::fl;

use crate::core::system_info::{CpuDynamicInfo, CpuStaticInfo, SystemInfo};
use cosmic::{app::Task, cosmic_theme, iced, theme, widget, Element};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextMenuAction {
    MulticoreView(bool),
}

impl widget::menu::Action for ContextMenuAction {
    type Message = AppMessage;
    fn message(&self) -> Self::Message {
        match self {
            ContextMenuAction::MulticoreView(visible) => AppMessage::MulticoreView(visible.clone()),
        }
    }
}

pub struct CpuResources {
    sys_info: Arc<RwLock<SystemInfo>>,

    cpu_usage_history: VecDeque<f32>,
    core_usage_history: Vec<VecDeque<f32>>,

    cpu_name: String,
    uptime: usize,
    cpu_usage: u32,
    current_frequency: u64,
    process_count: usize,
    thread_count: usize,
    descriptor_count: usize,

    multicore_view: bool,
}

impl super::Page for CpuResources {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::SysInfoRefresh => {
                if let Ok(sys) = self.sys_info.clone().read() {
                    let cpu = sys.cpu_dynamic_info();
                    let cpu_static = sys.cpu_static_info();
                    self.update_usage_graphs(&cpu);
                    self.update_metrics(&cpu, &cpu_static);
                }
            }
            AppMessage::MulticoreView(checked) => {
                self.multicore_view = checked;
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

impl CpuResources {
    pub fn new(sys_info: Arc<RwLock<SystemInfo>>) -> Self {
        Self {
            cpu_usage_history: VecDeque::from([0.0; 60]),
            core_usage_history: Vec::new(),

            cpu_name: String::from(""),
            uptime: 0,
            cpu_usage: 0,
            current_frequency: 0,
            process_count: 0,
            thread_count: 0,
            descriptor_count: 0,
            multicore_view: false,

            sys_info,
        }
    }

    fn update_usage_graphs(&mut self, cpu: &CpuDynamicInfo) {
        // Cpu Usage History Graph
        self.cpu_usage_history
            .push_back(cpu.overall_utilization_percent / 100.0);
        if self.cpu_usage_history.len() > 60 {
            self.cpu_usage_history.pop_front();
        }

        // Core usage history
        let cpus = &cpu.per_logical_cpu_utilization_percent;
        if self.core_usage_history.len() == 0 {
            for _ in 0..cpus.len() {
                self.core_usage_history.push(VecDeque::from([0.0; 60]));
            }
        }
        for i in 0..cpus.len() {
            self.core_usage_history[i].push_back(cpus[i] / 100.);
            if self.core_usage_history[i].len() > 60 {
                self.core_usage_history[i].pop_front();
            }
        }
    }

    fn update_metrics(&mut self, cpu: &CpuDynamicInfo, cpu_static: &CpuStaticInfo) {
        self.cpu_name = cpu_static.name.to_string();
        self.uptime = cpu.uptime_seconds as usize;
        self.cpu_usage = cpu.overall_utilization_percent as u32;
        self.current_frequency = cpu.current_frequency_mhz;
        self.process_count = cpu.process_count as usize;
        self.thread_count = cpu.thread_count as usize;
        self.descriptor_count = cpu.handle_count as usize;
    }

    fn context_menu(&self) -> Option<Vec<widget::menu::Tree<AppMessage>>> {
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

    fn graph(&self) -> Element<AppMessage> {
        // Usage graph
        let element = if self.multicore_view {
            let mut grid = widget::column().width(iced::Length::Fill);
            let mut row = widget::row().width(iced::Length::Fill).spacing(10.0);

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
                                .width(iced::Length::Fill),
                            ),
                    )
                    .width(iced::Length::FillPortion(1)),
                );

                if (i + 1) % 4 == 0 {
                    grid = grid.push(row);
                    grid = grid.push(widget::Space::with_height(5));
                    row = widget::row().width(iced::Length::Fill).spacing(10.0);
                }
            }

            grid = grid.push(row);

            widget::layer_container(widget::scrollable(grid)).width(iced::Length::Fill)
        } else {
            widget::layer_container(
                widget::canvas(crate::widgets::LineGraph {
                    steps: 59,
                    points: self.cpu_usage_history.clone(),
                    autoscale: false,
                })
                .height(iced::Length::Fill)
                .width(iced::Length::Fill),
            )
            .width(iced::Length::Fill)
        }
        .layer(cosmic_theme::Layer::Primary);

        let widget = cosmic::widget::context_menu(element, self.context_menu());
        widget.into()
    }

    fn info_column(&self, cosmic: &theme::CosmicTheme) -> Element<AppMessage> {
        widget::layer_container(
            widget::column()
                .spacing(cosmic.space_s())
                // Name
                .push(widget::text::title4(self.cpu_name.clone()))
                .push(iced::widget::horizontal_rule(1))
                // Utilization
                .push(
                    widget::row()
                        .align_y(iced::Alignment::Center)
                        .push(widget::text::heading(fl!("utilization")))
                        .push(widget::horizontal_space())
                        .push(widget::text::heading(format!("{}%", self.cpu_usage))),
                )
                // Speed
                .push(
                    widget::row()
                        .align_y(iced::Alignment::Center)
                        .push(widget::text::heading(fl!("speed")))
                        .push(widget::horizontal_space())
                        .push(widget::text::heading(format!(
                            "{}",
                            get_hz(self.current_frequency)
                        ))),
                )
                // Processes
                .push(
                    widget::row()
                        .align_y(iced::Alignment::Center)
                        .push(widget::text::body(fl!("processes")))
                        .push(widget::horizontal_space())
                        .push(widget::text::body(format!("{}", self.process_count))),
                )
                .push(
                    widget::row()
                        .align_y(iced::Alignment::Center)
                        .push(widget::text::body(fl!("threads")))
                        .push(widget::horizontal_space())
                        .push(widget::text::body(format!("{}", self.thread_count))),
                )
                .push(
                    widget::row()
                        .align_y(iced::Alignment::Center)
                        .push(widget::text::body(fl!("handles")))
                        .push(widget::horizontal_space())
                        .push(widget::text::body(format!("{}", self.descriptor_count))),
                ),
        )
        .layer(cosmic_theme::Layer::Primary)
        .width(iced::Length::Fixed(280.))
        .height(iced::Length::Fill)
        .padding([cosmic.space_s(), cosmic.space_m()])
        .into()

        // col = col.push(
        //     widget::row::with_children(vec![
        //         // CPU Utilization
        //         widget::column::with_children(vec![
        //             widget::text::heading(fl!("utilization")).into(),
        //             iced::widget::horizontal_rule(1).into(),
        //             widget::text::body(format!("{}%", self.cpu_usage)).into(),
        //         ])
        //         .spacing(cosmic.space_xxxs())
        //         .into(),
        //         // CPU Core Speed Average
        //         widget::column::with_children(vec![
        //             widget::text::heading(fl!("cpu-speed")).into(),
        //             iced::widget::horizontal_rule(1).into(),
        //             widget::text::body(get_hz(self.current_frequency)).into(),
        //         ])
        //         .spacing(cosmic.space_xxxs())
        //         .into(),
        //     ])
        //     .spacing(cosmic.space_xxs()),
        // );
        //
        // col = col.push(
        //     widget::row::with_children(vec![
        //         // Process count
        //         widget::column::with_children(vec![
        //             widget::text::heading("Processes").into(),
        //             iced::widget::horizontal_rule(1).into(),
        //             widget::text::body(format!("{}", self.process_count)).into(),
        //         ])
        //         .spacing(cosmic.space_xxxs())
        //         .into(),
        //         // Thread count
        //         widget::column::with_children(vec![
        //             widget::text::heading("Threads").into(),
        //             iced::widget::horizontal_rule(1).into(),
        //             widget::text::body(format!("{}", self.thread_count)).into(),
        //         ])
        //         .spacing(cosmic.space_xxxs())
        //         .into(),
        //         // Handles count
        //         widget::column::with_children(vec![
        //             widget::text::heading("Descriptors").into(),
        //             iced::widget::horizontal_rule(1).into(),
        //             widget::text::body(format!("{}", self.descriptor_count)).into(),
        //         ])
        //         .spacing(cosmic.space_xxxs())
        //         .into(),
        //     ])
        //     .spacing(cosmic.space_xxs()),
        // );
        //
        // let uptime = self.uptime;
        // let seconds = uptime % 60;
        // let minutes = (uptime / 60) % 60;
        // let hours = (uptime / 60) / 60;
        // col = col.push(
        //     widget::column::with_children(vec![
        //         widget::text::heading("Up time").into(),
        //         iced::widget::horizontal_rule(1).into(),
        //         widget::text::body(format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)).into(),
        //     ])
        //     .spacing(cosmic.space_xxxs()),
        // );
        //
        // widget::container(
        //     col.width(iced::Length::Fixed(256.))
        //         .height(iced::Length::Fill)
        //         .spacing(cosmic.space_s())
        //         .padding([cosmic.space_xs(), cosmic.space_xs()]),
        // )
        // .class(theme::Container::Primary)
        // .into()
    }
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
