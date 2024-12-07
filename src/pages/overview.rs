mod statistic;

use std::collections::HashMap;
use std::sync::Arc;
use crate::app::message::AppMessage;
use crate::core::icons;
use statistic::Statistic;

use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Background, Length};
use cosmic::{theme, widget, Element, Task};
use crate::system_info::{App, SystemInfo};

pub struct OverviewPage {
    // nothing yet
    statistics: Vec<Statistic>,
    applications: HashMap<Arc<str>, App>,
    selected_app: Option<u32>,
}

impl super::Page for OverviewPage {
    fn update(
        &mut self,
        sys: &SystemInfo,
        message: crate::app::message::AppMessage,
    ) -> cosmic::app::Task<AppMessage> {
        let mut tasks = Vec::new();
        match message {
            AppMessage::Refresh => {
                self.statistics.clear();
                self.statistics.push(Statistic::new(
                    "CPU".into(),
                    "processor-symbolic".into(),
                    sys.cpu_dynamic_info().overall_utilization_percent / 100.0,
                ));
                let mem_info = crate::system_info::mem_info::MemInfo::load().unwrap();
                let used = mem_info
                    .mem_total
                    .saturating_sub(mem_info.mem_available + mem_info.dirty) as f64 / mem_info.mem_total as f64;

                self.statistics.push(Statistic::new(
                    "RAM".into(),
                    "memory-symbolic".into(),
                    used as f32,
                ));
                let disks = sys.disks_info();
                for disk in disks.iter() {
                    self.statistics.push(Statistic::new(
                        format!("Disk {}", disk.model),
                        "harddisk-symbolic".into(),
                        disk.busy_percent / 100.0,
                    ));
                }
                self.applications = sys.apps().into();
            }
            AppMessage::ApplicationSelect(app) => {
                self.selected_app = app;
            }
            _ => {}
        }
        Task::batch(tasks)
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let theme = theme::active();
        let cosmic = theme.cosmic();

        let mut apps = widget::column()
            .spacing(cosmic.space_xxs());
        let mut applications = self.applications.values().collect::<Vec<&App>>();
        applications.sort_by_key(|a| &a.name);
        for app in  applications.into_iter().collect::<Vec<&App>>() {
            let is_selected = if let Some(selected_app) = self.selected_app {
                selected_app == app.pids[0]
            } else {
                false
            };
            apps = apps.push(
                widget::button::custom(
                    widget::row::with_children(vec![
                        widget::icon::from_name(app.icon.clone().unwrap()).size(24).into(),
                        widget::text::body(String::from(app.name.clone().as_ref())).into()
                    ])
                        .align_y(Vertical::Center)
                        .padding([cosmic.space_xxxs(), cosmic.space_xs()])
                        .spacing(cosmic.space_xs())
                        .width(Length::Fill),
                ).on_press(AppMessage::ApplicationSelect(Some(app.pids[0])))
                    .class(cosmic::style::Button::Custom {
                        active: Box::new(move |_, theme| {
                            let cosmic = theme.cosmic();
                            let mut appearance = widget::button::Style::new();
                            if is_selected {
                                appearance.background =
                                    Some(Background::Color(cosmic.accent.base.into()));
                                appearance.text_color = Some(cosmic.accent.on.into());
                            }
                            appearance.border_radius = cosmic.radius_s().into();
                            appearance
                        }),

                        disabled: Box::new(move |theme| {
                            let cosmic = theme.cosmic();
                            let mut appearance = widget::button::Style::new();
                            if is_selected {
                                appearance.background =
                                    Some(Background::Color(cosmic.accent.disabled.into()));
                                appearance.text_color = Some(cosmic.accent.on.into());
                            } else {
                                appearance.background =
                                    Some(Background::Color(cosmic.button.disabled.into()));
                                appearance.text_color = Some(cosmic.button.on_disabled.into());
                            }

                            appearance
                        }),
                        hovered: Box::new(move |_, theme| {
                            let cosmic = theme.cosmic();
                            let mut appearance = widget::button::Style::new();
                            if is_selected {
                                appearance.background =
                                    Some(Background::Color(cosmic.accent.hover.into()));
                                appearance.text_color = Some(cosmic.accent.on.into());
                            } else {
                                appearance.background =
                                    Some(Background::Color(cosmic.button.hover.into()));
                                appearance.text_color = Some(cosmic.button.on.into());
                            }
                            appearance.border_radius = cosmic.radius_s().into();
                            appearance
                        }),
                        pressed: Box::new(move |_, theme| {
                            let cosmic = theme.cosmic();
                            let mut appearance = widget::button::Style::new();
                            if is_selected {
                                appearance.background =
                                    Some(Background::Color(cosmic.accent.pressed.into()));
                                appearance.text_color = Some(cosmic.accent.on.into());
                            } else {
                                appearance.background =
                                    Some(Background::Color(cosmic.button.pressed.into()));
                                appearance.text_color = Some(cosmic.button.on.into());
                            }
                            appearance.border_radius = cosmic.radius_s().into();
                            appearance
                        }),
                    })
            )
        }

        widget::column::with_children(vec![
            widget::container(
                widget::row::with_children(
                    self.statistics
                        .iter()
                        .map(|statistic| {
                            widget::column::with_children(vec![
                                widget::tooltip(
                                    icons::get_icon(&statistic.icon, 18),
                                    widget::text(&statistic.name),
                                    widget::tooltip::Position::Bottom,
                                )
                                .into(),
                                widget::container(
                                    widget::canvas(crate::widgets::Meter {
                                        percentage: statistic.percent,
                                        thickness: 20.,
                                    })
                                    .width(Length::Fixed(100.))
                                    .height(Length::Fixed(100.)),
                                )
                                .padding([cosmic.space_xs(), cosmic.space_xs()])
                                .into(),
                            ])
                            .spacing(cosmic.space_xs())
                            .align_x(Horizontal::Center)
                            .width(Length::Fill)
                            .into()
                        })
                        .collect(),
                )
                .height(Length::Shrink)
                .padding([cosmic.space_l(), 0, 0, 0]),
            )
            .class(cosmic::style::Container::Primary)
            .into(),
            widget::layer_container(
                widget::column()
                    .push(widget::text::heading("Applications"))
                    .push(cosmic::iced_widget::horizontal_rule(1))
                    .push(widget::scrollable(apps))
                    .spacing(cosmic.space_xs()))
                .layer(cosmic::cosmic_theme::Layer::Primary)
                .padding([cosmic.space_xs(), cosmic.space_s()])
                .into()
        ])
        .spacing(cosmic.space_xs())
        .into()
    }
}

impl OverviewPage {
    pub fn new() -> Self {
        Self {
            statistics: Vec::new(),
            applications: HashMap::new(),
            selected_app: None,
        }
    }
}
