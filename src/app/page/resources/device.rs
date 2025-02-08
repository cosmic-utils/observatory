use crate::app::Message;
use cosmic::{iced, prelude::*, widget};
use std::borrow::Cow;

pub struct DeviceResource {
    label: Cow<'static, str>,
    graphs: widget::nav_bar::Model,

    on_next: Option<Message>,
    on_prev: Option<Message>,

    info: Vec<(Cow<'static, str>, Cow<'static, str>)>,
    statistics: Vec<(Cow<'static, str>, Cow<'static, str>)>,
}

impl DeviceResource {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            on_next: None,
            on_prev: None,
            graphs: widget::nav_bar::Model::default(),
            info: vec![],
            statistics: vec![],
        }
    }

    pub fn on_next(&mut self, msg: Message) {
        self.on_next = Some(msg);
    }

    pub fn on_prev(&mut self, msg: Message) {
        self.on_prev = Some(msg);
    }

    pub fn activate_graph(&mut self, index: u16) {
        self.graphs.activate_position(index);
    }

    pub fn activate_tab(&mut self, tab: widget::nav_bar::Id) {
        self.graphs.activate(tab);
    }

    pub fn contains_tab(&self, tab: widget::nav_bar::Id) -> bool {
        self.graphs.contains_item(tab)
    }

    pub fn add_graph(
        &mut self,
        label: impl Into<Cow<'static, str>>,
        graph: crate::widget::graph::LineGraph,
    ) {
        self.graphs.insert().text(label.into().clone()).data(graph);
    }

    pub fn map_graph(&mut self, label: impl Into<Cow<'static, str>>, value: f32) {
        let label = label.into();
        for entity in self.graphs.iter().collect::<Vec<widget::nav_bar::Id>>() {
            if self.graphs.text(entity).unwrap() == label.as_ref() {
                self.graphs.data_set(entity, value);
            }
        }
    }

    pub fn push_graph(&mut self, label: impl Into<Cow<'static, str>>, value: f32) {
        let label = label.into();
        let graph = self
            .graphs
            .iter()
            .find(|graph| self.graphs.text(*graph) == Some(label.as_ref()));
        if let Some(entity) = graph {
            if let Some(graph) = self
                .graphs
                .data_mut::<crate::widget::graph::LineGraph>(entity)
            {
                graph.points.push(value);
                graph.points = graph.points.iter().cloned().skip(1).collect();
            }
        }
    }

    pub fn set_statistic(
        &mut self,
        label: impl Into<Cow<'static, str>>,
        data: impl Into<Cow<'static, str>>,
    ) {
        let label = label.into();
        let data = data.into();
        if let Some(stat) = self
            .statistics
            .iter_mut()
            .find(|(ilabel, _)| *ilabel == label)
        {
            stat.1 = data.clone();
        } else {
            self.statistics.push((label.clone(), data.clone()));
        }
    }

    pub fn add_info(
        &mut self,
        label: impl Into<Cow<'static, str>>,
        data: impl Into<Cow<'static, str>>,
    ) {
        let label = label.into();
        let data = data.into();
        if let Some(info) = self.info.iter_mut().find(|(ilabel, _)| *ilabel == label) {
            info.1 = data;
        } else {
            self.info.push((label, data));
        }
    }

    pub fn get_info(&self, label: impl Into<Cow<'static, str>>) -> Option<Cow<'static, str>> {
        let label = label.into();
        self.info
            .iter()
            .find(|(ilabel, _)| *ilabel == label)
            .map(|found| found.1.clone())
    }

    pub fn view(&self) -> Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();
        widget::row()
            .height(iced::Length::Shrink)
            .spacing(cosmic.space_xxs())
            .push(
                widget::column()
                    .width(iced::Length::Fill)
                    .spacing(cosmic.space_xs())
                    .push_maybe(if self.graphs.iter().count() > 1 {
                        Some(
                            widget::tab_bar::horizontal(&self.graphs).on_activate(|entity| {
                                Message::ResourcePage(super::ResourceMessage::SelectDeviceTab(
                                    entity,
                                ))
                            }),
                        )
                    } else {
                        None
                    })
                    .push(widget::responsive(|size| {
                        self.graphs
                            .active_data::<crate::widget::graph::LineGraph>()
                            .unwrap()
                            .apply(|graph| {
                                if let Some(scale) = self.graphs.active_data::<f32>() {
                                    crate::widget::graph::LineGraph {
                                        points: graph
                                            .points
                                            .iter()
                                            .cloned()
                                            .map(|point| point / scale)
                                            .collect(),
                                    }
                                } else {
                                    graph.clone()
                                }
                            })
                            .apply(widget::canvas)
                            .height(size.width)
                            .width(size.width)
                            .apply(widget::container)
                            .width(iced::Length::Fill)
                            .height(iced::Length::Shrink)
                            .apply(Element::from)
                    })),
            )
            .push(
                widget::column()
                    .spacing(cosmic.space_xs())
                    .push_maybe(if !self.label.is_empty() {
                        Some(
                            widget::column()
                                .push(
                                    widget::row()
                                        .align_y(iced::Alignment::Center)
                                        .push(widget::text::title4(self.label.clone()))
                                        .push(widget::horizontal_space())
                                        .push(
                                            widget::button::icon(widget::icon::from_name(
                                                "go-previous-symbolic",
                                            ))
                                            .on_press_maybe(self.on_prev.clone()),
                                        )
                                        .push(
                                            widget::button::icon(widget::icon::from_name(
                                                "go-next-symbolic",
                                            ))
                                            .on_press_maybe(self.on_next.clone()),
                                        ),
                                )
                                .push(widget::divider::horizontal::default()),
                        )
                    } else {
                        None
                    })
                    .push(widget::settings::view_column(vec![
                        widget::settings::section()
                            .title("Statistics")
                            .extend(self.statistics.iter().map(|(label, value)| {
                                widget::settings::item(
                                    label.clone(),
                                    value.clone().apply(widget::text::body),
                                )
                            }))
                            .apply(Element::from),
                        widget::settings::section()
                            .title("Information")
                            .extend(self.info.iter().map(|(label, value)| {
                                widget::settings::item(
                                    label.clone(),
                                    value.clone().apply(widget::text::body),
                                )
                            }))
                            .apply(Element::from),
                    ])),
            )
            .apply(Element::from)
    }
}
