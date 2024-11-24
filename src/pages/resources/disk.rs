use crate::app::message::Message;
use crate::fl;

use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Length;
use cosmic::iced_widget::horizontal_rule;
use cosmic::{theme, widget, Element, Task};
use std::collections::VecDeque;

pub struct DiskResources {
    disk_write_history: VecDeque<f32>,
    disk_read_history: VecDeque<f32>,

    disk_read: u64,
    total_disk_read: u64,
    disk_write: u64,
    total_disk_write: u64,
}

impl super::Page for DiskResources {
    fn update(&mut self, sys: &sysinfo::System, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                let read_sum: u64 = sys
                    .processes()
                    .iter()
                    .map(|p| p.1.disk_usage().read_bytes)
                    .sum();
                let write_sum: u64 = sys
                    .processes()
                    .iter()
                    .map(|p| p.1.disk_usage().written_bytes)
                    .sum();

                self.disk_read = read_sum;
                self.disk_write = write_sum;
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
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = theme::active();
        let cosmic = theme.cosmic();
        let mem_name = widget::container(
            widget::text::title3(fl!("disk"))
                .width(Length::Fill)
                .height(Length::Shrink)
                .align_x(Horizontal::Left)
                .align_y(Vertical::Center),
        );

        let page = widget::row::with_children::<Message>(vec![
            widget::column()
                .push(widget::text::text(fl!("read")))
                .push(self.read_graph())
                .push(widget::text::text(fl!("write")))
                .push(self.write_graph())
                .into(),
            self.info_column(&cosmic),
        ])
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

impl DiskResources {
    pub fn new() -> Self {
        Self {
            disk_write_history: VecDeque::from([0.0; 60]),
            disk_read_history: VecDeque::from([0.0; 60]),

            disk_read: 0,
            total_disk_read: 0,
            disk_write: 0,
            total_disk_write: 0,
        }
    }

    fn read_graph(&self) -> Element<Message> {
        // Usage graph
        widget::container(
            widget::canvas(crate::widgets::line_graph::LineGraph {
                steps: 59,
                points: self.disk_read_history.clone(),
                autoscale: true,
            })
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }

    fn write_graph(&self) -> Element<Message> {
        // Usage graph
        widget::container(
            widget::canvas(crate::widgets::line_graph::LineGraph {
                steps: 59,
                points: self.disk_write_history.clone(),
                autoscale: true,
            })
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }

    fn info_column(&self, cosmic: &theme::CosmicTheme) -> Element<Message> {
        let mut col = widget::column::with_capacity(10);
        col = col.push(
            widget::row::with_children(vec![
                widget::column::with_children(vec![
                    widget::text::heading(fl!("total-read")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(self.total_disk_read),)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                widget::column::with_children(vec![
                    widget::text::heading(fl!("total-write")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(self.total_disk_write),)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
            ])
            .spacing(cosmic.space_xxs()),
        );

        col = col.push(
            widget::row::with_children(vec![
                widget::column::with_children(vec![
                    widget::text::heading(fl!("read")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(self.disk_read),)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                widget::column::with_children(vec![
                    widget::text::heading(fl!("write")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(self.disk_write),)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
            ])
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
