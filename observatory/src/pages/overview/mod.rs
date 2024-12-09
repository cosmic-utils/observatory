mod statistic;

use crate::app::message::AppMessage;
use crate::core::icons;
use statistic::Statistic;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::core::system_info::{App, SystemInfo};
use crate::fl;
use cosmic::{cosmic_theme, iced, theme, widget, Element, Task};

pub struct OverviewPage {
    // nothing yet
    statistics: Vec<Statistic>,
    applications: HashMap<Arc<str>, App>,
    selected_app: Option<String>,
    sys: Arc<RwLock<SystemInfo>>,
}

impl OverviewPage {
    pub fn new(system_info: Arc<RwLock<SystemInfo>>) -> Self {
        Self {
            statistics: Vec::new(),
            applications: HashMap::new(),
            selected_app: None,
            sys: system_info,
        }
    }
}

impl super::Page for OverviewPage {
    fn update(&mut self, message: AppMessage) -> cosmic::app::Task<AppMessage> {
        match message {
            AppMessage::SysInfoRefresh => {
                self.statistics.clear();
                if let Ok(sys) = self.sys.read() {
                    // CPU Usage statistic
                    let cpu_static = sys.cpu_static_info();
                    let cpu_dynamic = sys.cpu_dynamic_info();
                    self.statistics.push(Statistic::new(
                        "CPU".into(),
                        "processor-symbolic".into(),
                        cpu_dynamic.overall_utilization_percent / 100.0,
                        String::from(&*cpu_static.name),
                    ));

                    // Memory usage statistic
                    let mem_info = crate::core::system_info::mem_info::MemInfo::load().unwrap();
                    let used = mem_info
                        .mem_total
                        .saturating_sub(mem_info.mem_available + mem_info.dirty)
                        as f64
                        / mem_info.mem_total as f64;
                    self.statistics.push(Statistic::new(
                        "RAM".into(),
                        "memory-symbolic".into(),
                        used as f32,
                        format!("{} GiB", mem_info.mem_total / 1024usize.pow(3)),
                    ));

                    // GPU usage statistic
                    let gpu_static = sys.gpu_static_info();
                    let gpu_dynamic = sys.gpu_dynamic_info();
                    for i in 0..gpu_static.len() {
                        self.statistics.push(Statistic::new(
                            format!("GPU {}", i),
                            "processor-symbolic".into(),
                            gpu_dynamic[i].util_percent as f32 / 100.0,
                            String::from(&*gpu_static[i].device_name),
                        ))
                    }

                    // Disk usage statistic
                    let disks = sys.disks_info();
                    for i in 0..disks.len() {
                        self.statistics.push(Statistic::new(
                            format!("Disk {}", i),
                            "harddisk-symbolic".into(),
                            disks[i].busy_percent / 100.0,
                            String::from(&*disks[i].model),
                        ));
                    }
                    self.applications = sys.apps().into();
                }
            }
            AppMessage::OverviewApplicationSelect(app) => {
                self.selected_app = app;
            }
            AppMessage::OverviewApplicationClose => {
                if let Some(selected_app) = &self.selected_app {
                    if let Ok(sys) = self.sys.read() {
                        let app_name: Arc<str> = Arc::from(selected_app.as_ref());
                        if let Some(app) = sys.apps().get(&app_name) {
                            for pid in app.pids.iter() {
                                sys.kill_process(*pid);
                            }
                        }
                    }
                }
                self.selected_app = None;
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let theme = theme::active();
        let cosmic = theme.cosmic();

        // STATISTICS
        let mut statistics = widget::row().spacing(cosmic.space_m());
        for statistic in self.statistics.iter().cloned() {
            let stat_widget = widget::column()
                .spacing(cosmic.space_xxxs())
                .align_x(iced::Alignment::Center)
                .width(iced::Length::Fill)
                // Title
                .push(widget::tooltip(
                    widget::row()
                        .spacing(cosmic.space_xxs())
                        .push(icons::get_icon(statistic.icon.clone(), 18))
                        .push(widget::text(String::clone(&statistic.name))),
                    widget::text::body(String::clone(&statistic.hint)),
                    widget::tooltip::Position::Bottom,
                ))
                // Meter
                .push(
                    widget::container(
                        widget::canvas(crate::widgets::Meter {
                            percentage: statistic.percent,
                            thickness: 20.0,
                        })
                        .width(iced::Length::Fixed(100.0))
                        .height(iced::Length::Fixed(100.0)),
                    )
                    .padding([cosmic.space_xs(), cosmic.space_xs()]),
                );
            statistics = statistics.push(stat_widget);
        }

        let statistic_section = widget::column()
            .spacing(cosmic.space_xs())
            .push(widget::text::title4(fl!("resource-overview")))
            .push(iced::widget::horizontal_rule(1))
            .push(statistics);

        // APPLICATIONS
        let mut applications = self.applications.values().collect::<Vec<&App>>();
        let mut apps_list = widget::column().spacing(cosmic.space_xxs());
        applications.sort_by_key(|a| &a.name);
        for app in applications.into_iter().collect::<Vec<&App>>() {
            let is_selected = if let Some(selected_app) = &self.selected_app {
                *selected_app == app.id.to_string()
            } else {
                false
            };

            let app_widget = widget::row()
                .align_y(iced::Alignment::Center)
                .spacing(cosmic.space_xxs())
                .width(iced::Length::Fill)
                // App icon
                .push(widget::icon::from_name(app.icon.clone().unwrap().to_string()).size(24))
                // App name
                .push(widget::text::body(app.name.clone().to_string()));

            apps_list = apps_list.push(
                widget::button::custom(app_widget)
                    .on_press(AppMessage::OverviewApplicationSelect(Some(
                        app.id.to_string(),
                    )))
                    .class(crate::style::button::ButtonStyle::ListElement(is_selected).into()),
            )
        }
        let app_section = widget::column()
            .spacing(cosmic.space_xs())
            .push(widget::text::title4(fl!("applications")))
            .push(iced::widget::horizontal_rule(1))
            .push(widget::scrollable(apps_list));

        widget::column()
            .spacing(cosmic.space_xxs())
            .push(
                widget::layer_container(statistic_section)
                    .layer(cosmic_theme::Layer::Primary)
                    .height(iced::Length::Shrink)
                    .padding([cosmic.space_s(), cosmic.space_m()]),
            )
            .push(
                widget::layer_container(app_section)
                    .layer(cosmic_theme::Layer::Primary)
                    .height(iced::Length::Fill)
                    .padding([cosmic.space_s(), cosmic.space_m()]),
            )
            .into()
    }

    fn footer(&self) -> Option<Element<AppMessage>> {
        if let Some(_) = &self.selected_app {
            let theme = theme::active();
            let cosmic = theme.cosmic();
            Some(
                widget::layer_container(
                    widget::row().push(widget::horizontal_space()).push(
                        widget::button::suggested("Close")
                            .on_press(AppMessage::OverviewApplicationClose),
                    ),
                )
                .padding([cosmic.space_xxs(), cosmic.space_xxs()])
                .layer(cosmic_theme::Layer::Primary)
                .into(),
            )
        } else {
            None
        }
    }
}
