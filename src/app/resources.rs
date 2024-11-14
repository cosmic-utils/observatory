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


pub struct ResourcePage {
    tab_model: widget::segmented_button::SingleSelectModel,
    active_page: TabPage,
    cpu_id: raw_cpuid::CpuId<raw_cpuid::CpuIdReaderNative>,
    cpu_usages: VecDeque<f32>,
}

impl ResourcePage {
    pub fn new(sys: &sysinfo::System) -> Self {
        let mut tab_model = widget::segmented_button::SingleSelectModel::default();
        tab_model.insert().text("CPU").data(TabPage::Cpu);
        tab_model.insert().text("Memory").data(TabPage::Memory);
        tab_model.insert().text("Disk").data(TabPage::Disk);
        tab_model.activate_position(0);


        Self {
            tab_model,
            active_page: TabPage::Cpu,
            cpu_id: raw_cpuid::CpuId::new(),
            cpu_usages: VecDeque::new(),
        }
    }

    pub fn update(&mut self, sys: &sysinfo::System, message: Message) {
        match message {
            Message::ResourceTabSelected(entity) => {
                self.tab_model.activate(entity);
                self.active_page = self.tab_model.active_data::<TabPage>().unwrap().clone();
            }
            Message::Refresh => {
                self.cpu_usages.push_front(sys.global_cpu_usage() / sys.cpus().len() as f32);
                if self.cpu_usages.len() > 60 {
                    self.cpu_usages.pop_back();
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
        };

        widget::column::with_children(vec![tabs.into(), page_data.into()]).into()
    }

    fn cpu(&self, theme: &theme::Theme, sys: &sysinfo::System) -> iced_widget::Container<Message, theme::Theme> {
        let cosmic = theme.cosmic();
        let page = widget::row::with_children::<Message>(vec![
            widget::column::with_children(vec![
                widget::text::title4(self.cpu_id.get_processor_brand_string().unwrap().as_str().to_string())
                    .width(Length::Fill).height(Length::Fixed(48.))
                    .align_x(Horizontal::Left)
                    .align_y(Vertical::Center)
                    .into(),
                widget::container(widget::canvas(crate::widget::line_graph::LineGraph { steps: 59, points: self.cpu_usages.clone() })
                    .width(Length::Fixed(400.0))
                    .height(Length::Fixed(400.0))
                )
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .padding([cosmic.space_xs(), cosmic.space_xs()])
                    .into()
            ])
                .padding([0, cosmic.space_xs()])
                .width(Length::FillPortion(2))
                .into(),
            widget::text::heading("CPU INFO WIP")
                .width(Length::FillPortion(1))
                .into(),
        ]);

        widget::container(page)
            .height(Length::Fill)
    }
    fn mem(&self, sys: &sysinfo::System) -> iced_widget::Container<Message, theme::Theme> {
        widget::container(widget::text::heading("Mem Information TODO"))
    }
    fn disk(&self, sys: &sysinfo::System) -> iced_widget::Container<Message, theme::Theme> {
        widget::container(widget::text::heading("Disk Information TODO"))
    }
}

#[derive(Clone, Debug)]
enum TabPage {
    Cpu,
    Memory,
    Disk,
}
