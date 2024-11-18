use crate::app::message::Message;
use crate::core::icons;
use crate::fl;
use std::collections::VecDeque;

use cosmic::iced_widget::horizontal_rule;
use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    iced_widget, theme, widget, Element,
};

pub struct ResourcePage {
    tab_model: widget::segmented_button::SingleSelectModel,
    active_page: TabPage,
    cpu_id: raw_cpuid::CpuId<raw_cpuid::CpuIdReaderNative>,
    cpu_usages: VecDeque<f32>,
    cores_usages: Vec<VecDeque<f32>>,
    mem_usages: VecDeque<f32>,
    write_disk_usages: VecDeque<f32>,
    read_disk_usages: VecDeque<f32>,
    total_disk_read: u64,
    total_disk_write: u64,
    multicore_view: bool,
}

impl ResourcePage {
    pub fn new() -> Self {
        let mut tab_model = widget::segmented_button::SingleSelectModel::default();
        tab_model
            .insert()
            .text(format!(" {}", fl!("cpu")))
            .data(TabPage::Cpu)
            .icon(icons::get_icon("processor-symbolic", 18));
        tab_model
            .insert()
            .text(format!(" {}", fl!("memory")))
            .data(TabPage::Memory)
            .icon(icons::get_icon("memory-symbolic", 18));
        tab_model
            .insert()
            .text(format!(" {}", fl!("disk")))
            .data(TabPage::Disk)
            .icon(icons::get_icon("harddisk-symbolic", 18));
        tab_model.activate_position(0);

        Self {
            tab_model,
            active_page: TabPage::Cpu,
            cpu_id: raw_cpuid::CpuId::new(),
            cpu_usages: VecDeque::from([0.0; 60]),
            cores_usages: Vec::new(),
            mem_usages: VecDeque::from([0.0; 60]),
            write_disk_usages: VecDeque::from([0.0; 60]),
            read_disk_usages: VecDeque::from([0.0; 60]),
            total_disk_read: 0,
            total_disk_write: 0,
            multicore_view: false,
        }
    }

    pub fn update(&mut self, sys: &sysinfo::System, message: Message) {
        match message {
            Message::ResourceTabSelected(entity) => {
                self.tab_model.activate(entity);
                self.active_page = self.tab_model.active_data::<TabPage>().unwrap().clone();
            }
            Message::Refresh => {
                self.cpu_usages.push_back(sys.global_cpu_usage() / 100.);
                if self.cpu_usages.len() > 60 {
                    self.cpu_usages.pop_front();
                }

                let cpus = sys.cpus();
                if self.cores_usages.len() == 0 {
                    for _ in 0..cpus.len() {
                        self.cores_usages.push(VecDeque::from([0.0; 60]));
                    }
                }
                if cpus.len() > 0 {
                    for i in 0..cpus.len() {
                        self.cores_usages[i].push_back(cpus[i].cpu_usage() / 100.);
                        if self.cores_usages[i].len() > 60 {
                            self.cores_usages[i].pop_front();
                        }
                    }
                }

                self.mem_usages.push_back(
                    calc_usage_percentage(sys.used_memory(), sys.total_memory()) as f32 / 100.,
                );
                if self.mem_usages.len() > 60 {
                    self.mem_usages.pop_front();
                }
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
                self.total_disk_read += read_sum;
                self.total_disk_write += write_sum;
                self.write_disk_usages.push_back(write_sum as f32);
                if self.write_disk_usages.len() > 60 {
                    self.write_disk_usages.pop_front();
                }
                self.read_disk_usages.push_back(read_sum as f32);
                if self.read_disk_usages.len() > 60 {
                    self.read_disk_usages.pop_front();
                }
            }
            Message::MulticoreView(checked) => {
                self.multicore_view = checked;
            }
            _ => {}
        }
    }

    pub fn view(&self, sys: &sysinfo::System) -> Element<Message> {
        let theme = theme::active();

        // Tab bar
        let tabs = widget::segmented_button::horizontal(&self.tab_model)
            .style(theme::SegmentedButton::TabBar)
            .button_alignment(Alignment::Center)
            .maximum_button_width(50)
            .on_activate(Message::ResourceTabSelected);

        // Data
        let page_data = match self.active_page {
            TabPage::Cpu => self.cpu(&theme, sys),
            TabPage::Memory => self.mem(&theme, sys),
            TabPage::Disk => self.disk(&theme, sys),
        }
        .width(Length::Fill);

        widget::column::with_children(vec![tabs.into(), page_data.into()]).into()
    }

    fn cpu_graph(&self) -> Element<Message> {
        // Usage graph
        if self.multicore_view {
            let mut grid = widget::column().width(Length::Fill);
            let mut row = widget::row().width(Length::Fill).spacing(10.0);

            for (i, usage) in self.cores_usages.iter().enumerate() {
                row = row.push(
                    widget::container(
                        widget::column()
                            .push(widget::text::text(format!("{} {}", fl!("core"), i)))
                            .push(
                                widget::canvas(crate::widget::LineGraph {
                                    steps: 59,
                                    points: usage.clone(),
                                    autoscale: false,
                                })
                                .width(Length::Fill),
                            ),
                    )
                    .width(Length::FillPortion(1)),
                );

                if (i + 1) % 4 == 0 {
                    grid = grid.push(row);
                    grid = grid.push(widget::Space::with_height(5));
                    row = widget::row().width(Length::Fill).spacing(10.0);
                }
            }

            grid = grid.push(row);

            widget::scrollable(grid).into()
        } else {
            widget::container(
                widget::canvas(crate::widget::LineGraph {
                    steps: 59,
                    points: self.cpu_usages.clone(),
                    autoscale: false,
                })
                .height(Length::Fill)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .into()
        }
    }

    fn cpu_info_column(&self, theme: &theme::Theme, sys: &sysinfo::System) -> Element<Message> {
        let cosmic = theme.cosmic();
        let mut col = widget::column::with_capacity(10);
        let (_, _, avg) = get_cpu_freqs(sys);
        col = col.push(
            widget::row::with_children(vec![
                // CPU Utilization
                widget::column::with_children(vec![
                    widget::text::heading(fl!("utilization")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}%", sys.global_cpu_usage() as u32)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                // CPU Core Speed Average
                widget::column::with_children(vec![
                    widget::text::heading(fl!("speed-avg")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(get_hz(avg)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
            ])
            .spacing(cosmic.space_xxs()),
        );

        col = col.push(
            widget::row::with_children(vec![
                // Process count
                widget::column::with_children(vec![
                    widget::text::heading("Processes").into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!(
                        "{}",
                        sys.processes()
                            .iter()
                            .filter(|proc| proc.1.thread_kind().is_none())
                            .count()
                    ))
                    .into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                // Thread count
                widget::column::with_children(vec![
                    widget::text::heading("Threads").into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", sys.processes().len())).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                // Handles count
                widget::column::with_children(vec![
                    widget::text::heading("Handles").into(),
                    horizontal_rule(1).into(),
                    widget::text::body("TODO").into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
            ])
            .spacing(cosmic.space_xxs()),
        );

        let uptime = sysinfo::System::uptime();
        let seconds = uptime % 60;
        let minutes = (uptime / 60) % 60;
        let hours = (uptime / 60) / 60;
        col = col.push(
            widget::column::with_children(vec![
                widget::text::heading("Up time").into(),
                horizontal_rule(1).into(),
                widget::text::body(format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)).into(),
            ])
            .spacing(cosmic.space_xxxs()),
        );

        col = col.push(
            widget::checkbox(fl!("core-view"), self.multicore_view)
                .on_toggle(Message::MulticoreView),
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

    fn mem_info_column(&self, theme: &theme::Theme, sys: &sysinfo::System) -> Element<Message> {
        let cosmic = theme.cosmic();
        let mut col = widget::column::with_capacity(10);
        col = col.push(
            widget::row::with_children(vec![
                // CPU Utilization
                widget::column::with_children(vec![
                    widget::text::heading(fl!("total-mem")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(sys.total_memory()))).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                // CPU Core Speed Average
                widget::column::with_children(vec![
                    widget::text::heading(fl!("total-swap")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(sys.total_swap()))).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
            ])
            .spacing(cosmic.space_xxs()),
        );

        col = col.push(
            widget::row::with_children(vec![widget::column::with_children(vec![
                widget::text::heading(fl!("mem-utilization")).into(),
                horizontal_rule(1).into(),
                widget::text::body(format!(
                    "{} ({:.1}%)",
                    format_size(sys.used_memory()),
                    calc_usage_percentage(sys.used_memory(), sys.total_memory())
                ))
                .into(),
            ])
            .spacing(cosmic.space_xxxs())
            .into()])
            .spacing(cosmic.space_xxs()),
        );

        col = col.push(
            widget::row::with_children(vec![widget::column::with_children(vec![
                widget::text::heading(fl!("swap-utilization")).into(),
                horizontal_rule(1).into(),
                widget::text::body(format!(
                    "{} ({:.1}%)",
                    format_size(sys.used_swap()),
                    calc_usage_percentage(sys.used_swap(), sys.total_swap())
                ))
                .into(),
            ])
            .spacing(cosmic.space_xxxs())
            .into()])
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

    fn cpu(
        &self,
        theme: &theme::Theme,
        sys: &sysinfo::System,
    ) -> iced_widget::Container<Message, theme::Theme> {
        let cosmic = theme.cosmic();
        let cpu_name = widget::container(
            widget::text::title3(format!(
                "Processor  —  {}",
                self.cpu_id
                    .get_processor_brand_string()
                    .unwrap()
                    .as_str()
                    .to_string()
            ))
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center),
        );
        let page = widget::row::with_children::<Message>(vec![
            self.cpu_graph(),
            self.cpu_info_column(theme, sys),
        ])
        .spacing(cosmic.space_s());

        widget::container(
            widget::column::with_children(vec![
                cpu_name.into(),
                horizontal_rule(1).into(),
                page.into(),
            ])
            .spacing(cosmic.space_xs()),
        )
        .padding([cosmic.space_xxs(), cosmic.space_xxs()])
        .width(Length::Fill)
        .height(Length::Fill)
    }

    fn mem_graph(&self) -> Element<Message> {
        // Usage graph
        widget::container(
            widget::canvas(crate::widget::line_graph::LineGraph {
                steps: 59,
                points: self.mem_usages.clone(),
                autoscale: false,
            })
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }

    fn mem(
        &self,
        theme: &theme::Theme,
        sys: &sysinfo::System,
    ) -> iced_widget::Container<Message, theme::Theme> {
        let cosmic = theme.cosmic();
        let mem_name = widget::container(
            widget::text::title3(fl!("memory"))
                .width(Length::Fill)
                .height(Length::Shrink)
                .align_x(Horizontal::Left)
                .align_y(Vertical::Center),
        );

        let page = widget::row::with_children::<Message>(vec![
            self.mem_graph(),
            self.mem_info_column(theme, sys),
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
    }

    fn disk(
        &self,
        theme: &theme::Theme,
        sys: &sysinfo::System,
    ) -> iced_widget::Container<Message, theme::Theme> {
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
                .push(self.read_disk_graph())
                .push(widget::text::text(fl!("write")))
                .push(self.write_disk_graph())
                .into(),
            self.disk_info_column(theme, sys),
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
    }

    fn disk_info_column(&self, theme: &theme::Theme, sys: &sysinfo::System) -> Element<Message> {
        let cosmic = theme.cosmic();
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

        col = col.push(
            widget::row::with_children(vec![
                widget::column::with_children(vec![
                    widget::text::heading(fl!("read")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(read_sum),)).into(),
                ])
                .spacing(cosmic.space_xxxs())
                .into(),
                widget::column::with_children(vec![
                    widget::text::heading(fl!("write")).into(),
                    horizontal_rule(1).into(),
                    widget::text::body(format!("{}", format_size(write_sum),)).into(),
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

    fn write_disk_graph(&self) -> Element<Message> {
        // Usage graph
        widget::container(
            widget::canvas(crate::widget::line_graph::LineGraph {
                steps: 59,
                points: self.write_disk_usages.clone(),
                autoscale: true,
            })
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }

    fn read_disk_graph(&self) -> Element<Message> {
        // Usage graph
        widget::container(
            widget::canvas(crate::widget::line_graph::LineGraph {
                steps: 59,
                points: self.read_disk_usages.clone(),
                autoscale: true,
            })
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }
}

#[derive(Clone, Debug)]
enum TabPage {
    Cpu,
    Memory,
    Disk,
}

fn get_cpu_freqs(sys: &sysinfo::System) -> (u64, u64, u64) {
    let mut total: u64 = 0;
    let mut max: u64 = 0;
    let mut min: u64 = u64::MAX;
    for cpu in sys.cpus() {
        total = total + cpu.frequency();
        if cpu.frequency() < min {
            min = cpu.frequency();
        }
        if cpu.frequency() > max {
            max = cpu.frequency();
        }
    }
    let avg = total / sys.cpus().len() as u64;
    (min, max, avg)
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

fn calc_usage_percentage(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64) * 100.0
    }
}
