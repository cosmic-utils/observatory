use crate::fl;
use cosmic::{
    app::{context_drawer, Task},
    iced::Length,
    prelude::*,
    widget,
};
use monitord::system::Process;

use crate::{
    app::{ContextPage, Message},
    config::Config,
};

#[derive(Clone, Debug)]
pub enum ProcessMessage {
    SelectProcess(widget::table::Entity),
    SortCategory(ProcessTableCategory),
}

pub struct ProcessPage {
    process_model: widget::table::SingleSelectModel<ProcessTableItem, ProcessTableCategory>,
    show_info: bool,
    // Configuration data that persists between application runs.
    config: Config,
}

impl ProcessPage {
    pub fn new(config: Config) -> Self {
        Self {
            process_model: widget::table::SingleSelectModel::new(vec![
                ProcessTableCategory::Name,
                ProcessTableCategory::Cpu,
                ProcessTableCategory::Gpu(0),
                ProcessTableCategory::Mem,
                ProcessTableCategory::Disk,
            ]),
            show_info: false,
            config,
        }
    }
}

impl super::Page for ProcessPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        let tasks = Vec::new();
        match msg {
            Message::UpdateConfig(config) => self.config = config,
            Message::Snapshot(snapshot) => {
                let old_sort = self.process_model.get_sort();
                let active_process = self
                    .process_model
                    .item(self.process_model.active())
                    .map(|process| process.process.pid);
                self.process_model.clear();
                for process in snapshot.processes.iter().cloned().map(|mut process| {
                    if !self.config.scale_by_core {
                        process.cpu /= snapshot.cpu_static_info.logical_cores as f32;
                    }
                    process
                }) {
                    self.process_model
                        .insert(ProcessTableItem {
                            process: process.clone(),
                        })
                        .apply(|entity| {
                            if let Some(pid) = active_process {
                                if process.pid == pid {
                                    entity.activate();
                                }
                            }
                        });
                }
                if let Some(sort) = old_sort {
                    self.process_model.sort(sort.0, sort.1);
                } else {
                    self.process_model.sort(ProcessTableCategory::Name, false)
                }
            }
            Message::ProcessPageMessage(msg) => match msg {
                ProcessMessage::SelectProcess(process) => self.process_model.activate(process),
                ProcessMessage::SortCategory(category) => {
                    if let Some(sort) = self.process_model.get_sort() {
                        if sort.0 == category {
                            self.process_model.sort(category, !sort.1);
                        } else {
                            self.process_model.sort(category, false)
                        }
                    } else {
                        self.process_model.sort(category, false)
                    }
                }
            },
            Message::ToggleContextPage(page) => {
                if let ContextPage::PageAbout = page {
                    self.show_info = true;
                }
            }

            _ => {}
        }

        Task::batch(tasks)
    }

    fn view(&self) -> Element<Message> {
        widget::table(&self.process_model)
            .on_item_left_click(|entity| {
                Message::ProcessPageMessage(ProcessMessage::SelectProcess(entity))
            })
            .on_category_left_click(|cat| {
                Message::ProcessPageMessage(ProcessMessage::SortCategory(cat))
            })
            .apply(widget::scrollable)
            .apply(widget::container)
            .height(Length::Fill)
            .apply(Element::from)
    }

    fn footer(&self) -> Option<Element<Message>> {
        if self
            .process_model
            .item(self.process_model.active())
            .is_some()
        {
            widget::row()
                .push(widget::horizontal_space())
                .push(
                    fl!("details")
                        .to_string()
                        .apply(widget::button::text)
                        .on_press(Message::ToggleContextPage(ContextPage::PageAbout)),
                )
                .apply(widget::layer_container)
                .layer(cosmic::cosmic_theme::Layer::Primary)
                .apply(Element::from)
                .apply(Some)
        } else {
            None
        }
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Message>> {
        if let Some(selected) = self.process_model.item(self.process_model.active()) {
            let process = &selected.process;
            Some(context_drawer::context_drawer(
                widget::settings::section()
                    .title(fl!("proc-info"))
                    .add(widget::settings::item(
                        fl!("internal-name"),
                        widget::text::caption(process.name.clone()),
                    ))
                    .add(widget::settings::item(
                        fl!("cmd-line"),
                        widget::text::caption(format!("{}", process.cmd.join(" "))),
                    ))
                    .add(widget::settings::item(
                        fl!("exe"),
                        widget::text::caption(process.exe.clone()),
                    ))
                    .apply(Element::from),
                Message::ToggleContextPage(ContextPage::PageAbout),
            ))
        } else {
            None
        }
    }
}

struct ProcessTableItem {
    process: Process,
}

impl widget::table::ItemInterface<ProcessTableCategory> for ProcessTableItem {
    fn get_icon(&self, category: ProcessTableCategory) -> Option<widget::Icon> {
        match category {
            ProcessTableCategory::Name => {
                Some(widget::icon::from_name("applications-system-symbolic").icon())
            }
            _ => None,
        }
    }

    fn get_text(&self, category: ProcessTableCategory) -> std::borrow::Cow<'static, str> {
        match category {
            ProcessTableCategory::Name => self.process.displayname.clone().into(),
            ProcessTableCategory::Cpu => format!("{}%", self.process.cpu.round()).into(),
            ProcessTableCategory::Gpu(num) => {
                format!("{}%", self.process.gpu[num as usize].round()).into()
            }
            ProcessTableCategory::Mem => get_bytes(self.process.memory).into(),
            ProcessTableCategory::Disk => format!("{}/s", get_bytes(self.process.disk)).into(),
        }
    }

    fn compare(&self, other: &Self, category: ProcessTableCategory) -> std::cmp::Ordering {
        match category {
            ProcessTableCategory::Name => other
                .process
                .displayname
                .to_ascii_lowercase()
                .cmp(&self.process.displayname.to_ascii_lowercase()),
            ProcessTableCategory::Cpu => self.process.cpu.partial_cmp(&other.process.cpu).unwrap(),
            ProcessTableCategory::Gpu(num) => self.process.gpu[num as usize]
                .partial_cmp(&other.process.gpu[num as usize])
                .unwrap(),
            ProcessTableCategory::Mem => self.process.memory.cmp(&other.process.memory),
            ProcessTableCategory::Disk => self.process.disk.cmp(&other.process.disk),
        }
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ProcessTableCategory {
    #[default]
    Name,
    Cpu,
    Gpu(u16),
    Mem,
    Disk,
}

impl std::fmt::Display for ProcessTableCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Name => fl!("name"),
                Self::Cpu => fl!("cpu"),
                Self::Gpu(num) => fl!("gpu", num = num),
                Self::Mem => fl!("mem"),
                Self::Disk => fl!("disk"),
            }
        )
    }
}

impl widget::table::ItemCategory for ProcessTableCategory {
    fn width(&self) -> cosmic::iced::Length {
        match self {
            Self::Name => Length::Fixed(320.0),
            Self::Cpu => Length::Fixed(80.0),
            Self::Gpu(_) => Length::Fixed(80.0),
            Self::Mem => Length::Fixed(120.0),
            Self::Disk => Length::Fixed(150.0),
        }
    }
}

fn get_bytes(bytes: u64) -> String {
    if bytes < 1024u64.pow(1) {
        format!("{} B", bytes)
    } else if bytes < 1024u64.pow(2) {
        format!("{:.2} KiB", bytes as f64 / 1024f64.powf(1.))
    } else if bytes < 1024u64.pow(3) {
        format!("{:.2} MiB", bytes as f64 / 1024f64.powf(2.))
    } else {
        format!("{:.2} GiB", bytes as f64 / 1024f64.powf(3.))
    }
}
