use crate::app::message::AppMessage;
use crate::fl;

use crate::core::system_info::{DiskInfo, SystemInfo};
use cosmic::{app::Task, cosmic_theme, iced, theme, widget, Element};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

pub struct DiskResources {
    disk_write_history: VecDeque<f32>,
    disk_read_history: VecDeque<f32>,

    disks: Vec<DiskInfo>,

    total_disk_read: u64,
    total_disk_write: u64,

    sys: Arc<RwLock<SystemInfo>>,
}

impl super::Page for DiskResources {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::SysInfoRefresh => {
                if let Ok(sys) = self.sys.read() {
                    self.disks = sys.disks_info();

                    let read_sum = self
                        .disks
                        .iter()
                        .map(|disk_info| disk_info.read_speed)
                        .sum::<u64>();
                    let write_sum = self
                        .disks
                        .iter()
                        .map(|disk_info| disk_info.write_speed)
                        .sum::<u64>();

                    self.total_disk_read += read_sum;
                    self.total_disk_write += write_sum;
                    self.disk_write_history.push_back(write_sum as f32);
                    if self.disk_write_history.len() > 60 {
                        self.disk_write_history.pop_front();
                    }
                    self.disk_read_history.push_back(read_sum as f32);
                    if self.disk_read_history.len() > 60 {
                        self.disk_read_history.pop_front();
                    }
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

impl DiskResources {
    pub fn new(sys: Arc<RwLock<SystemInfo>>) -> Self {
        Self {
            disk_write_history: VecDeque::from([0.0; 60]),
            disk_read_history: VecDeque::from([0.0; 60]),

            disks: Vec::new(),

            total_disk_read: 0,
            total_disk_write: 0,

            sys,
        }
    }

    fn graph(&self) -> Element<'_, AppMessage> {
        widget::column()
            .push(widget::text::text(fl!("read")))
            .push(self.read_graph())
            .push(widget::text::text(fl!("write")))
            .push(self.write_graph())
            .into()
    }

    fn read_graph(&self) -> Element<AppMessage> {
        // Usage graph
        widget::layer_container(
            widget::canvas(crate::widgets::line_graph::LineGraph {
                steps: 59,
                points: self.disk_read_history.clone(),
                autoscale: true,
            })
            .height(iced::Length::Fill)
            .width(iced::Length::Fill),
        )
        .layer(cosmic_theme::Layer::Primary)
        .width(iced::Length::Fill)
        .into()
    }

    fn write_graph(&self) -> Element<AppMessage> {
        // Usage graph
        widget::layer_container(
            widget::canvas(crate::widgets::line_graph::LineGraph {
                steps: 59,
                points: self.disk_write_history.clone(),
                autoscale: true,
            })
            .height(iced::Length::Fill)
            .width(iced::Length::Fill),
        )
        .layer(cosmic_theme::Layer::Primary)
        .width(iced::Length::Fill)
        .into()
    }

    fn info_column(&self, cosmic: &theme::CosmicTheme) -> Element<AppMessage> {
        let mut disks = widget::column().spacing(cosmic.space_s());
        for disk_info in &self.disks {
            disks = disks
                .push(widget::text::title4(format!("{}", disk_info.model)))
                .push(iced::widget::horizontal_rule(1))
                .push(
                    widget::row()
                        .push(widget::text::heading(fl!("capacity")))
                        .push(widget::horizontal_space())
                        .push(widget::text::heading(format_size(disk_info.capacity))),
                )
                .push(
                    widget::row()
                        .push(widget::text::body(fl!("read")))
                        .push(widget::horizontal_space())
                        .push(widget::text::body(format_speed(disk_info.read_speed))),
                )
                .push(
                    widget::row()
                        .push(widget::text::body(fl!("write")))
                        .push(widget::horizontal_space())
                        .push(widget::text::body(format_speed(disk_info.write_speed))),
                )
        }

        widget::layer_container(widget::scrollable(
            widget::column().spacing(cosmic.space_s()).push(disks),
        ))
        .layer(cosmic_theme::Layer::Primary)
        .width(iced::Length::Fixed(280.))
        .height(iced::Length::Fill)
        .padding([cosmic.space_s(), cosmic.space_m()])
        .into()
    }
}

fn format_speed(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if size >= TB {
        format!("{:.1} TiB/s", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.1} GiB/s", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MiB/s", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KiB/s", size as f64 / KB as f64)
    } else {
        format!("{} B/s", size)
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

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
