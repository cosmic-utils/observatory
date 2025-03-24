mod cpu;
mod device;
mod disk;
mod gpu;
mod mem;

use device::DeviceResource;
use monitord_protocols::monitord::*;

use crate::{app::Message, config::Config, fl};
use cosmic::{
    app::Task,
    iced::Subscription,
    prelude::*,
    widget::{self, nav_bar},
};

#[derive(Debug, Clone)]
pub enum ResourceMessage {
    CpuSnapshot(CpuInfo),
    MemorySnapshot(MemoryInfo),
    GpuSnapshot(GpuList),
    DiskSnapshot(StorageList),

    SelectTab(widget::segmented_button::Entity),
    SelectDeviceTab(widget::segmented_button::Entity),
    DiskNext,
    DiskPrev,
    GpuNext,
    GpuPrev,
}

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
                        .data(Box::new(cpu::CpuPage::new(config.clone())) as Box<dyn super::Page>)
                        .activate()
                })
                .insert(|b| {
                    b.text(fl!("mem"))
                        .icon(widget::icon::from_name("firmware-manager-symbolic"))
                                                .data(Box::new(mem::MemoryPage::new(config.clone()))
                                                    as Box<dyn super::Page>)
                })
                .insert(|b| {
                    b.text("GPU")
                        .icon(widget::icon::from_name("firmware-manager-symbolic"))
                        .data(Box::new(gpu::GpuPage::new(config.clone())) as Box<dyn super::Page>)
                })
                .insert(|b| {
                    b.text(fl!("disk"))
                        .icon(widget::icon::from_name("drive-harddisk-system-symbolic"))
                        .data(Box::new(disk::DiskPage::new(config.clone())) as Box<dyn super::Page>)
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
            Message::ResourcePage(ResourceMessage::SelectTab(tab)) => self.tabs.activate(tab),
            Message::UpdateConfig(config) => self.config = config,
            _ => {}
        }
        for page in self
            .tabs
            .iter()
            .collect::<Vec<widget::segmented_button::Entity>>()
        {
            if let Some(tab) = self.tabs.data_mut::<Box<dyn super::Page>>(page) {
                tasks.push(tab.update(msg.clone()));
            }
        }
        Task::batch(tasks)
    }

    fn view(&self) -> Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();
        widget::column()
            .spacing(cosmic.space_s())
            .push(
                widget::tab_bar::horizontal(&self.tabs)
                    .on_activate(|entity| Message::ResourcePage(ResourceMessage::SelectTab(entity)))
                    .button_spacing(cosmic.space_xxs()),
            )
            .push(
                if let Some(tab) = self.tabs.active_data::<Box<dyn super::Page>>() {
                    tab.view()
                } else {
                    widget::horizontal_space().apply(Element::from)
                },
            )
            .apply(Element::from)
    }

    fn subscription(&self) -> Vec<Subscription<Message>> {
        let mut subscriptions = Vec::new();

        let entities = self.tabs.iter().collect::<Vec<nav_bar::Id>>();
        for entity in entities {
            let page = self.tabs.data::<Box<dyn super::Page>>(entity);
            if let Some(page) = page {
                subscriptions.push(Subscription::batch(page.subscription()));
            }
        }

        subscriptions
    }
}
