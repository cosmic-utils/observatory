use std::collections::VecDeque;

use cosmic::{app::Task, iced, prelude::*, widget};

use crate::{
    app::{page::Page, Message},
    config::Config,
};

pub struct DiskPage {
    //
    config: Config,

    read_history: VecDeque<f32>,
    write_history: VecDeque<f32>,
}

impl DiskPage {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            read_history: vec![0.0; 30].into(),
            write_history: vec![0.0; 30].into(),
        }
    }
}

impl Page for DiskPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Snapshot(_snapshot) => {
                // todo
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
                .push(widget::settings::view_column(vec![]))
                .apply(Element::from)
        })
        .apply(Element::from)
    }
}
