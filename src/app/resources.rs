use std::collections::VecDeque;
use crate::app::message::Message;

use cosmic::{iced::{Length,
                    Alignment,
                    alignment::{
                        Vertical,
                        Horizontal,
                    }},
             iced_widget,
             theme,
             widget,
             Element};
use cosmic::iced_widget::horizontal_rule;

pub struct ResourcePage {
    tab_model: widget::segmented_button::SingleSelectModel,
    active_page: TabPage,
    cpu_id: raw_cpuid::CpuId<raw_cpuid::CpuIdReaderNative>,
    cpu_usages: VecDeque<f32>,
    mem_usages: VecDeque<f32>,
}

impl ResourcePage {
    pub fn new() -> Self {
        let mut tab_model = widget::segmented_button::SingleSelectModel::default();
        tab_model.insert().text("CPU").data(TabPage::Cpu);
        tab_model.insert().text("Memory").data(TabPage::Memory);
        tab_model.insert().text("Disk").data(TabPage::Disk);
        tab_model.activate_position(0);


        Self {
            tab_model,
            active_page: TabPage::Cpu,
            cpu_id: raw_cpuid::CpuId::new(),
            cpu_usages: VecDeque::from([0.0; 60]),
            mem_usages: VecDeque::from([0.0; 60]),
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
            TabPage::Memory => self.mem(sys),
            TabPage::Disk => self.disk(sys),
        }.width(Length::Fill);

        widget::column::with_children(vec![tabs.into(), page_data.into()]).into()
    }

    fn cpu_graph(&self) -> Element<Message> {
        // Usage graph
        widget::container(widget::canvas(crate::widget::LineGraph { steps: 59, points: self.cpu_usages.clone() })
            .height(Length::Fill)
            .width(Length::Fill)
        )
            .width(Length::Fill)
            .into()
    }

    fn cpu_info_column(&self, theme: &theme::Theme, sys: &sysinfo::System) -> Element<Message> {
        let cosmic = theme.cosmic();
        let mut col = widget::column::with_capacity(10);
        let (_, _, avg) = get_cpu_freqs(sys);
        col = col.push(widget::row::with_children(vec![
            // CPU Utilization
            widget::column::with_children(vec![
                widget::text::heading("Utilization").into(),
                horizontal_rule(1).into(),
                widget::text::body(format!("{}%", sys.global_cpu_usage() as u32)).into(),
            ]).spacing(cosmic.space_xxxs()).into(),
            // CPU Core Speed Average
            widget::column::with_children(vec![
                widget::text::heading("Speed (Core Avg)").into(),
                horizontal_rule(1).into(),
                widget::text::body(get_hz(avg)).into()
            ]).spacing(cosmic.space_xxxs()).into()
        ]).spacing(cosmic.space_xxs()));

        col = col.push(widget::row::with_children(vec![
            // Process count
            widget::column::with_children(vec![
                widget::text::heading("Processes").into(),
                horizontal_rule(1).into(),
                widget::text::body(format!("{}", sys.processes().iter().filter(|proc| proc.1.thread_kind().is_none()).count())).into()
            ]).spacing(cosmic.space_xxxs()).into(),
            // Thread count
            widget::column::with_children(vec![
                widget::text::heading("Threads").into(),
                horizontal_rule(1).into(),
                widget::text::body(format!("{}", sys.processes().len())).into()
            ]).spacing(cosmic.space_xxxs()).into(),
            // Handles count
            widget::column::with_children(vec![
                widget::text::heading("Handles").into(),
                horizontal_rule(1).into(),
                widget::text::body("TODO").into()
            ]).spacing(cosmic.space_xxxs()).into()
        ]).spacing(cosmic.space_xxs()));

        let uptime = sysinfo::System::uptime();
        let seconds = uptime % 60;
        let minutes = (uptime / 60) % 60;
        let hours = (uptime / 60) / 60;
        col = col.push(widget::column::with_children(vec![
            widget::text::heading("Up time").into(),
            horizontal_rule(1).into(),
            widget::text::body(format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)).into()
        ]).spacing(cosmic.space_xxxs()));


        widget::container(col
            .width(Length::Fixed(256.))
            .height(Length::Fill)
            .spacing(cosmic.space_s())
            .padding([cosmic.space_xs(), cosmic.space_xs()])
        )
            .class(theme::Container::Primary)
            .into()
    }

    fn cpu(&self, theme: &theme::Theme, sys: &sysinfo::System) -> iced_widget::Container<Message, theme::Theme> {
        let cosmic = theme.cosmic();
        let cpu_name = widget::container(
            widget::text::title3(format!("Processor  —  {}", self.cpu_id.get_processor_brand_string().unwrap().as_str().to_string()))
                .width(Length::Fill)
                .height(Length::Shrink)
                .align_x(Horizontal::Left)
                .align_y(Vertical::Center)
        );
        let page = widget::row::with_children::<Message>(vec![
            self.cpu_graph(),
            self.cpu_info_column(theme, sys),
        ])
            .spacing(cosmic.space_s());

        widget::container(widget::column::with_children(vec![
            cpu_name.into(), horizontal_rule(1).into(), page.into()
        ])
            .spacing(cosmic.space_xs())
        )
            .padding([cosmic.space_xxs(), cosmic.space_xxs()])
            .width(Length::Fill)
            .height(Length::Fill)
    }

    fn mem_graph(&self) -> Element<Message> {
        // Usage graph
        widget::container(widget::canvas(crate::widget::line_graph::LineGraph { steps: 59, points: self.mem_usages.clone() })
            .height(Length::Fill)
            .width(Length::Fill)
        )
            .width(Length::Fill)
            .into()
    }


    fn mem(&self, _sys: &sysinfo::System) -> iced_widget::Container<Message, theme::Theme> {
        widget::container(widget::text::heading("Mem Information TODO"))
    }
    fn disk(&self, _sys: &sysinfo::System) -> iced_widget::Container<Message, theme::Theme> {
        widget::container(widget::text::heading("Disk Information TODO"))
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
