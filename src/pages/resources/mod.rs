mod cpu;
mod mem;
//mod disk;

use crate::app::message::AppMessage;
use crate::core::icons;
use crate::fl;
use std::sync::{Arc, RwLock};

use crate::core::system_info::SystemInfo;
use crate::pages::Page;
use cosmic::{app::Task, iced, theme, widget, Element};

pub struct ResourcePage {
    tab_model: widget::segmented_button::SingleSelectModel,
}

impl ResourcePage {
    pub fn new(system_info: Arc<RwLock<SystemInfo>>) -> Self {
        let mut tab_model = widget::segmented_button::SingleSelectModel::default();
        tab_model
            .insert()
            .text(format!(" {}", fl!("cpu")))
            .data(Box::new(cpu::CpuResources::new(system_info.clone())) as Box<dyn Page>)
            .icon(icons::get_icon("processor-symbolic".into(), 18));
        tab_model
            .insert()
            .text(format!(" {}", fl!("memory")))
            .data(Box::new(mem::MemResources::new()) as Box<dyn Page>)
            .icon(icons::get_icon("memory-symbolic".into(), 18));
        // tab_model
        //     .insert()
        //     .text(format!(" {}", fl!("disk")))
        //     .data(Box::new(disk::DiskResources::new()) as Box<dyn Page>)
        //     .icon(icons::get_icon("harddisk-symbolic".into(), 18));
        tab_model.activate_position(0);

        Self { tab_model }
    }
}

impl Page for ResourcePage {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        let mut tasks = Vec::new();

        let entities = self
            .tab_model
            .iter()
            .collect::<Vec<widget::segmented_button::Entity>>();
        match message {
            AppMessage::ResourceTabSelected(entity) => {
                self.tab_model.activate(entity);
            }
            _ => {}
        }

        for entity in entities {
            let page = self.tab_model.data_mut::<Box<dyn Page>>(entity);
            if let Some(page) = page {
                tasks.push(page.update(message.clone()));
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let theme = theme::active();
        let cosmic = theme.cosmic();

        if let Some(page) = self.tab_model.active_data::<Box<dyn Page>>() {
            widget::column()
                .spacing(cosmic.space_xs())
                .push(
                    widget::segmented_button::horizontal(&self.tab_model)
                        .style(theme::SegmentedButton::TabBar)
                        .button_alignment(iced::Alignment::Center)
                        .on_activate(AppMessage::ResourceTabSelected),
                )
                .push(page.view())
                .into()
        } else {
            widget::text::heading("Error, unknown resource page!").into()
        }
    }
}
