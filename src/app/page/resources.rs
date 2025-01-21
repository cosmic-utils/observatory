mod cpu;

use crate::{app::Message, config::Config, fl};
use cosmic::{app::Task, prelude::*, widget};

pub struct ResourcePage {
    tabs: widget::segmented_button::SingleSelectModel,
    // Configuration data that persists between application runs.
    config: Config,
}

impl ResourcePage {
    pub fn new(config: Config) -> Self {
        Self {
            tabs: widget::segmented_button::SingleSelectModel::builder()
                .insert(|b| {
                    b.text(fl!("cpu"))
                        .icon(widget::icon::from_name("firmware-manager-symbolic"))
                        .data(Box::new(cpu::CpuPage::new()) as Box<dyn super::Page>)
                        .activate()
                })
                .build(),
            config,
        }
    }
}

impl super::Page for ResourcePage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        let mut tasks = Vec::new();
        match msg.clone() {
            Message::UpdateConfig(config) => self.config = config,
            _ => {}
        }
        for page in self
            .tabs
            .iter()
            .collect::<Vec<widget::segmented_button::Entity>>()
        {
            tasks.push(
                self.tabs
                    .data_mut::<Box<dyn super::Page>>(page)
                    .unwrap()
                    .update(msg.clone()),
            );
        }
        Task::batch(tasks)
    }

    fn view(&self) -> Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();
        widget::column()
            .spacing(cosmic.space_s())
            .push(widget::tab_bar::horizontal(&self.tabs))
            .push(
                self.tabs
                    .active_data::<Box<dyn super::Page>>()
                    .unwrap()
                    .view(),
            )
            .apply(Element::from)
    }
}
