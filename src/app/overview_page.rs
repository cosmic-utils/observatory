mod statistic;

use crate::app::message::Message;
use crate::core::icons;
use statistic::Statistic;

use cosmic::iced::alignment::Horizontal;
use cosmic::iced::Length;
use cosmic::{theme, widget, Element};
use sysinfo::Disks;

pub struct OverviewPage {
    // nothing yet
    statistics: Vec<Statistic>,
}

impl OverviewPage {
    pub fn new() -> Self {
        Self {
            statistics: Vec::new(),
        }
    }

    pub fn update(&mut self, sys: &sysinfo::System, _message: Message) {
        self.statistics.clear();
        self.statistics.push(Statistic::new(
            "CPU".into(),
            "processor-symbolic".into(),
            sys.global_cpu_usage() / 100.,
        ));
        self.statistics.push(Statistic::new(
            "RAM".into(),
            "memory-symbolic".into(),
            sys.used_memory() as f32 / sys.total_memory() as f32,
        ));
        let disks = Disks::new_with_refreshed_list();
        let mut i = 0;
        for disk in disks.list().iter().filter(|disk| {
            !disk.mount_point().to_string_lossy().contains("/boot")
                && !disk.mount_point().to_string_lossy().contains("/recovery")
        }) {
            if i > 1 {
                break;
            }
            self.statistics.push(Statistic::new(
                format!("Disk {}", i),
                "harddisk-symbolic".into(),
                (disk.total_space() - disk.available_space()) as f32 / disk.total_space() as f32,
            ));
            i = i + 1;
        }
    }

    pub fn view(&self, _sys: &sysinfo::System) -> Element<Message> {
        let theme = theme::active();
        let cosmic = theme.cosmic();

        widget::column::with_children(vec![widget::row::with_children(
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
                            widget::canvas(crate::widget::Meter {
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
        .padding([cosmic.space_l(), 0, 0, 0])
        .into()])
        .into()
    }
}
