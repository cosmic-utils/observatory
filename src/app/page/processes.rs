use cosmic::{
    app::{context_drawer, Task},
    iced::Length,
    prelude::*,
    widget,
};
use monitord::system::Process;

use crate::app::{ContextPage, Message};

pub struct ProcessPage {
    process_model: widget::table::SingleSelectModel<ProcessTableItem, ProcessTableCategory>,
    show_info: bool,
}

impl ProcessPage {
    pub fn new() -> Self {
        Self {
            process_model: widget::table::SingleSelectModel::new(vec![
                ProcessTableCategory::Name,
                ProcessTableCategory::Cpu,
                ProcessTableCategory::Mem,
                ProcessTableCategory::Disk,
            ]),
            show_info: false,
        }
    }
}

impl super::Page for ProcessPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        let tasks = Vec::new();
        match msg {
            Message::Snapshot(snapshot) => {
                let old_sort = self.process_model.get_sort();
                let active_process = self
                    .process_model
                    .item(self.process_model.active())
                    .map(|process| process.process.pid);
                self.process_model.clear();
                for process in snapshot.processes {
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
                }
            }
            Message::SelectProcess(process) => {
                self.process_model.activate(process);
            }
            Message::SortCategory(category) => {
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
            .on_item_left_click(Message::SelectProcess)
            .on_category_left_click(Message::SortCategory)
            .apply(widget::scrollable)
            .apply(widget::container)
            .height(Length::Fill)
            .apply(Element::from)
    }

    fn footer(&self) -> Option<Element<Message>> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();
        if self
            .process_model
            .item(self.process_model.active())
            .is_some()
        {
            widget::row()
                .push(widget::horizontal_space())
                .push(
                    "Details"
                        .to_string()
                        .apply(widget::button::text)
                        .on_press(Message::ToggleContextPage(ContextPage::PageAbout)),
                )
                .apply(widget::layer_container)
                .layer(cosmic::cosmic_theme::Layer::Primary)
                .padding([cosmic.space_xs(), cosmic.space_s()])
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
                    .title("Process Information")
                    .add(widget::settings::item(
                        "OS Name",
                        widget::text::caption(process.name.clone()),
                    ))
                    .add(widget::settings::item(
                        "Command Line",
                        widget::text::caption(format!("{:?}", process.cmd.clone())),
                    ))
                    .add(widget::settings::item(
                        "Executable",
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
    fn get_icon(&self, _: ProcessTableCategory) -> Option<widget::Icon> {
        None
    }

    fn get_text(&self, category: ProcessTableCategory) -> std::borrow::Cow<'static, str> {
        match category {
            ProcessTableCategory::Name => self.process.displayname.clone().into(),
            ProcessTableCategory::Cpu => format!("{}%", self.process.cpu.round()).into(),
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
    Mem,
    Disk,
}

impl std::fmt::Display for ProcessTableCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Name => "Name",
            Self::Cpu => "CPU",
            Self::Mem => "Memory",
            Self::Disk => "Disk",
        })
    }
}

impl widget::table::ItemCategory for ProcessTableCategory {
    fn width(&self) -> cosmic::iced::Length {
        match self {
            Self::Name => Length::Fixed(250.0),
            Self::Cpu => Length::Fixed(120.0),
            Self::Mem => Length::Fixed(150.0),
            Self::Disk => Length::Fixed(160.0),
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
