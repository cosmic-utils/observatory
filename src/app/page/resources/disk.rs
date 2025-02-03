use std::{borrow::Cow, collections::VecDeque};

use crate::fl;
use cosmic::{app::Task, iced, prelude::*, widget};
use lazy_static::lazy_static;
use monitord::system::disk::{DiskDynamic, DiskStatic};

lazy_static! {
    static ref NOT_LOADED: Cow<'static, str> = fl!("not-loaded").into();
    static ref DISK_STATS: Cow<'static, str> = fl!("disk-stats").into();
    static ref DISK_READ: Cow<'static, str> = fl!("disk-read").into();
    static ref DISK_WRITE: Cow<'static, str> = fl!("disk-write").into();
    static ref DISK_DEV: Cow<'static, str> = fl!("disk-dev").into();
    static ref DISK_CAP: Cow<'static, str> = fl!("disk-cap").into();
}

use crate::{
    app::{page::Page, Message},
    config::Config,
};

use super::ResourceMessage;

#[derive(PartialEq, Eq)]
enum ShowGraph {
    Write,
    Read,
}

pub struct DiskPage {
    tab: widget::segmented_button::SingleSelectModel,
    disk_info: Option<(Vec<DiskStatic>, DiskDynamic)>,

    config: Config,

    read_history: VecDeque<u64>,
    max_read: u64,
    write_history: VecDeque<u64>,
    max_write: u64,
}

impl DiskPage {
    pub fn new(config: Config) -> Self {
        Self {
            tab: widget::segmented_button::ModelBuilder::default()
                .insert(|b| b.text(DISK_READ.clone()).data(ShowGraph::Read).activate())
                .insert(|b| b.text(DISK_WRITE.clone()).data(ShowGraph::Write))
                .build(),
            disk_info: None,
            config,
            read_history: vec![0; 30].into(),
            max_read: 1,
            write_history: vec![0; 30].into(),
            max_write: 1,
        }
    }
}

impl Page for DiskPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Snapshot(snapshot) => {
                self.max_read = self.max_read.max(snapshot.disk.1.read);
                self.read_history.push_back(snapshot.disk.1.read);
                self.read_history.pop_front();
                self.max_write = self.max_write.max(snapshot.disk.1.write);
                self.write_history.push_back(snapshot.disk.1.write);
                self.write_history.pop_front();
                self.disk_info = Some(snapshot.disk.clone())
            }
            Message::UpdateConfig(config) => self.config = config,
            Message::ResourcePage(ResourceMessage::SelectDiskTab(entity)) => self.tab.activate(entity),
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
                    widget::column()
                        .spacing(cosmic.space_s())
                        .push(
                            widget::tab_bar::horizontal(&self.tab).on_activate(|entity| {
                                Message::ResourcePage(ResourceMessage::SelectDiskTab(entity))
                            }),
                        )
                        .push(
                            widget::canvas(crate::widget::graph::LineGraph {
                                points: if *self.tab.active_data::<ShowGraph>().unwrap()
                                    == ShowGraph::Read
                                {
                                    self.read_history
                                        .iter()
                                        .cloned()
                                        .map(|read| {
                                            (read as f64
                                                / self
                                                    .read_history
                                                    .iter()
                                                    .max()
                                                    .cloned()
                                                    .unwrap()
                                                    .max(1)
                                                    as f64)
                                                as f32
                                        })
                                        .collect::<Vec<f32>>()
                                } else {
                                    self.write_history
                                        .iter()
                                        .cloned()
                                        .map(|read| {
                                            (read as f64
                                                / self
                                                    .write_history
                                                    .iter()
                                                    .max()
                                                    .cloned()
                                                    .unwrap()
                                                    .max(1)
                                                    as f64)
                                                as f32
                                        })
                                        .collect::<Vec<f32>>()
                                },
                            })
                            .width(iced::Length::Fill)
                            .height(size.width.min(size.height))
                            .apply(widget::container),
                        ),
                )
                .push(
                    widget::settings::view_column(vec![widget::settings::section()
                        .title(DISK_STATS.clone())
                        .add(widget::settings::item(
                            DISK_READ.clone(),
                            self.disk_info
                                .as_ref()
                                .map(|(_, disk_stats)| {
                                    disk_stats
                                        .read
                                        .apply(crate::helpers::get_bytes)
                                        .apply(widget::text::body)
                                })
                                .unwrap_or(widget::text::body(NOT_LOADED.clone())),
                        ))
                        .add(widget::settings::item(
                            DISK_WRITE.clone(),
                            self.disk_info
                                .as_ref()
                                .map(|(_, disk_stats)| {
                                    disk_stats
                                        .write
                                        .apply(crate::helpers::get_bytes)
                                        .apply(widget::text::body)
                                })
                                .unwrap_or(widget::text::body(NOT_LOADED.clone())),
                        ))
                        .apply(Element::from)])
                    .extend(
                        self.disk_info
                            .as_ref()
                            .map(|disk_info| {
                                disk_info
                                    .0
                                    .iter()
                                    .map(|disk| {
                                        widget::settings::section()
                                            .title(disk.model.clone())
                                            .add(widget::settings::item(
                                                DISK_DEV.clone(),
                                                disk.device.clone().apply(widget::text::body),
                                            ))
                                            .add(widget::settings::item(
                                                DISK_CAP.clone(),
                                                disk.size
                                                    .apply(crate::helpers::get_bytes)
                                                    .apply(widget::text::body),
                                            ))
                                            .apply(Element::from)
                                    })
                                    .collect::<Vec<Element<Message>>>()
                            })
                            .unwrap_or_default(),
                    ),
                )
                .apply(Element::from)
        })
        .apply(Element::from)
    }
}
