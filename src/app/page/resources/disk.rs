use std::{borrow::Cow, collections::VecDeque};

use crate::fl;
use cosmic::{app::Task, iced, prelude::*, widget};
use lazy_static::lazy_static;
use monitord::system::disk::DiskStatic;

lazy_static! {
    static ref DISK_STATS: Cow<'static, str> = fl!("disk-stats");
    static ref DISK_DEV: Cow<'static, str> = fl!("disk-dev");
    static ref DISK_CAP: Cow<'static, str> = fl!("disk-cap");
}

use crate::{
    app::{page::Page, Message},
    config::Config,
};

pub struct DiskPage {
    disk_info: Option<Vec<DiskStatic>>,

    config: Config,

    read_history: VecDeque<f32>,
    write_history: VecDeque<f32>,
}

impl DiskPage {
    pub fn new(config: Config) -> Self {
        Self {
            disk_info: None,
            config,
            read_history: vec![0.0; 30].into(),
            write_history: vec![0.0; 30].into(),
        }
    }
}

impl Page for DiskPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Snapshot(snapshot) => {
                self.disk_info = Some(
                    snapshot
                        .disk
                        .iter()
                        .map(|(disk_static, _)| disk_static.clone())
                        .collect::<Vec<DiskStatic>>(),
                )
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
                .push(
                    widget::column()
                        .push(
                            widget::canvas(crate::widget::graph::LineGraph {
                                points: self.read_history.iter().cloned().collect::<Vec<f32>>(),
                            })
                            .width(size.width.min(size.height * 1.2))
                            .height(size.height.min(size.width * 1.2))
                            .apply(widget::container)
                            .width(iced::Length::Fill),
                        )
                        .push(
                            widget::canvas(crate::widget::graph::LineGraph {
                                points: self.write_history.iter().cloned().collect::<Vec<f32>>(),
                            })
                            .width(size.width.min(size.height * 1.2))
                            .height(size.height.min(size.width * 1.2))
                            .apply(widget::container)
                            .width(iced::Length::Fill),
                        ),
                )
                .push(
                    widget::settings::view_column(vec![widget::settings::section()
                        .title(DISK_STATS.clone())
                        .add("TODO".apply(widget::text::heading))
                        .apply(Element::from)])
                    .extend(
                        self.disk_info
                            .as_ref()
                            .map(|disk_info| {
                                disk_info
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
                            .unwrap_or(vec![]),
                    ),
                )
                .apply(Element::from)
        })
        .apply(Element::from)
    }
}
