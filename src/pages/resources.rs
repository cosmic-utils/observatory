mod cpu;
mod disk;
mod mem;

use crate::app::message::Message;
use crate::core::icons;
use crate::fl;

use crate::pages::Page;
use cosmic::{iced::Alignment, theme, widget, Element, Task};

pub struct ResourcePage {
    tab_model: widget::segmented_button::SingleSelectModel,
}

impl Page for ResourcePage {
    fn update(&mut self, sys: &sysinfo::System, message: Message) -> Task<Message> {
        let mut tasks = Vec::new();

        let entities = self
            .tab_model
            .iter()
            .collect::<Vec<widget::segmented_button::Entity>>();
        match message {
            Message::ResourceTabSelected(entity) => {
                self.tab_model.activate(entity);
            }
            _ => {}
        }

        for entity in entities {
            let page = self.tab_model.data_mut::<Box<dyn Page>>(entity);
            if let Some(page) = page {
                tasks.push(page.update(sys, message.clone()));
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Tab bar
        let tabs = widget::segmented_button::horizontal(&self.tab_model)
            .style(theme::SegmentedButton::TabBar)
            .button_alignment(Alignment::Center)
            .maximum_button_width(50)
            .on_activate(Message::ResourceTabSelected)
            .into();

        // Data
        let page_data = if let Some(page) = self.tab_model.active_data::<Box<dyn Page>>() {
            page.view()
        } else {
            widget::text::heading("Error, unknown resource page!").into()
        };

        widget::column::with_children(vec![tabs, page_data]).into()
    }
}

impl ResourcePage {
    pub fn new() -> Self {
        let mut tab_model = widget::segmented_button::SingleSelectModel::default();
        tab_model
            .insert()
            .text(format!(" {}", fl!("cpu")))
            .data(Box::new(cpu::CpuResources::new()) as Box<dyn Page>)
            .icon(icons::get_icon("processor-symbolic", 18));
        tab_model
            .insert()
            .text(format!(" {}", fl!("memory")))
            .data(Box::new(mem::MemResources::new()) as Box<dyn Page>)
            .icon(icons::get_icon("memory-symbolic", 18));
        tab_model
            .insert()
            .text(format!(" {}", fl!("disk")))
            .data(Box::new(disk::DiskResources::new()) as Box<dyn Page>)
            .icon(icons::get_icon("harddisk-symbolic", 18));
        tab_model.activate_position(0);

        Self { tab_model }
    }
}
