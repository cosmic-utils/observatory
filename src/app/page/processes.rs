mod process;
use process::{ProcessTableCategory, ProcessTableItem};

use cosmic::{
    app::{context_drawer, Task},
    iced::Length,
    prelude::*,
    widget,
};

use crate::{
    app::{ContextPage, Message},
    config::Config,
    fl,
};
#[derive(Clone, Debug)]
pub enum ProcessMessage {
    SelectProcess(widget::table::Entity),
    SortCategory(ProcessTableCategory),
    KillProcess(u32),
    TermProcess(u32),
}

pub struct ProcessPage {
    process_model: widget::table::SingleSelectModel<ProcessTableItem, ProcessTableCategory>,
    show_info: bool,
    // Configuration data that persists between application runs.
    config: Config,
    // Interface
    interface: Option<monitord::Interface<'static>>,
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
            interface: None,
        }
    }
}

impl super::Page for ProcessPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        let mut tasks = Vec::new();
        match msg {
            Message::UpdateConfig(config) => self.config = config,
            Message::InterfaceLoaded(interface) => self.interface = Some(interface),
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
                    let pid = process.pid;
                    let item = ProcessTableItem::new(process);
                    self.process_model.insert(item).apply(|entity| {
                        if let Some(active_pid) = active_process {
                            if pid == active_pid {
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
                ProcessMessage::KillProcess(pid) => match self.interface.clone() {
                    Some(interface) => tasks.push(Task::future(async move {
                        interface
                            .kill_process(pid)
                            .await
                            .expect("Failed to term process!");
                        cosmic::app::message::app(Message::NoOp)
                    })),
                    None => {}
                },
                ProcessMessage::TermProcess(pid) => match self.interface.clone() {
                    Some(interface) => tasks.push(Task::future(async move {
                        interface
                            .term_process(pid)
                            .await
                            .expect("Failed to term process!");
                        cosmic::app::message::app(Message::NoOp)
                    })),
                    None => {}
                },
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
            .id(widget::Id::new("PROCESS_SCROLLABLE"))
            .height(Length::Fill)
            .apply(Element::from)
    }

    fn footer(&self) -> Option<Element<Message>> {
        if self
            .process_model
            .item(self.process_model.active())
            .is_some()
        {
            let theme = cosmic::theme::active();
            let cosmic = theme.cosmic();
            widget::row()
                .push(widget::horizontal_space())
                .spacing(cosmic.space_xxxs())
                .padding([cosmic.space_xxxs(), cosmic.space_xxs()])
                .push(
                    fl!("details")
                        .to_string()
                        .apply(widget::button::text)
                        .on_press(Message::ToggleContextPage(ContextPage::PageAbout)),
                )
                .push(
                    fl!("kill")
                        .to_string()
                        .apply(widget::button::destructive)
                        .on_press(Message::ProcessPageMessage(ProcessMessage::KillProcess(
                            self.process_model
                                .item(self.process_model.active())
                                .unwrap()
                                .process
                                .pid,
                        ))),
                )
                .push(
                    fl!("term")
                        .to_string()
                        .apply(widget::button::suggested)
                        .on_press(Message::ProcessPageMessage(ProcessMessage::TermProcess(
                            self.process_model
                                .item(self.process_model.active())
                                .unwrap()
                                .process
                                .pid,
                        ))),
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
                    .add(widget::settings::item(
                        fl!("status"),
                        widget::text::caption(process.status.clone()),
                    ))
                    .apply(Element::from),
                Message::ToggleContextPage(ContextPage::PageAbout),
            ))
        } else {
            None
        }
    }
}
