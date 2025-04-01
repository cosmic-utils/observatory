mod process;
use futures_util::SinkExt;
use process::{ProcessTableCategory, ProcessTableItem};

use cosmic::{
    app::{context_drawer, Task},
    iced::{stream, Length, Subscription},
    prelude::*,
    widget,
};
use monitord_protocols::monitord::ProcessSig::{Sigkill, Sigterm};
use monitord_protocols::monitord::ProcessSigRequest;
use crate::{
    app::{ContextPage, Message},
    config::Config,
    fl,
};
#[derive(Clone, Debug)]
pub enum ProcessMessage {
    ProcessList(monitord_protocols::monitord::ProcessList),
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
}

impl ProcessPage {
    pub fn new(config: Config) -> Self {
        Self {
            process_model: widget::table::SingleSelectModel::new(vec![
                ProcessTableCategory::Name,
                ProcessTableCategory::Cpu,
                ProcessTableCategory::Gpu,
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
        let mut tasks = Vec::new();
        match msg {
            Message::UpdateConfig(config) => self.config = config,
            Message::ProcessPage(msg) => match msg {
                ProcessMessage::ProcessList(processes) => {
                    let old_sort = self.process_model.get_sort();
                    let active_process = self
                        .process_model
                        .item(self.process_model.active())
                        .map(|process| process.process.pid);
                    self.process_model.clear();
                    for process in processes.processes.iter().cloned() {
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
                ProcessMessage::KillProcess(pid) => {
                    tasks.push(Task::future(async move {
                        use monitord_protocols::protocols::MonitordServiceClient;
                        let mut client = MonitordServiceClient::connect("http://127.0.0.1:50051")
                            .await
                            .unwrap();

                        let request = tonic::Request::new(ProcessSigRequest {
                            pid,
                            sig: Sigkill.into()
                        });

                        let succeeded = client.term_process(request).await.unwrap().into_inner();

                        if succeeded.succeeded {
                            cosmic::Action::App(Message::NoOp)
                        } else {
                            cosmic::Action::App(Message::Error("Failed to kill process".to_owned()))
                        }
                    }));
                }
                ProcessMessage::TermProcess(pid) => {
                    tasks.push(Task::future(async move {
                        use monitord_protocols::protocols::MonitordServiceClient;
                        let mut client = MonitordServiceClient::connect("http://127.0.0.1:50051")
                            .await
                            .unwrap();

                        let request = tonic::Request::new(ProcessSigRequest {
                            pid,
                            sig: Sigterm.into()
                        });

                        let succeeded = client.term_process(request).await.unwrap().into_inner();

                        if succeeded.succeeded {
                            cosmic::Action::App(Message::NoOp)
                        } else {
                            cosmic::Action::App(Message::Error("Failed to term process".to_owned()))
                        }
                    }));
                }
            },
            Message::ToggleContextPage(ContextPage::PageAbout) => {
                self.show_info = true;
            }

            _ => {}
        }

        Task::batch(tasks)
    }

    fn view(&self) -> Element<Message> {
        widget::table(&self.process_model)
            .on_item_left_click(|entity| {
                Message::ProcessPage(ProcessMessage::SelectProcess(entity))
            })
            .on_category_left_click(|cat| Message::ProcessPage(ProcessMessage::SortCategory(cat)))
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
                        .apply(widget::button::text)
                        .on_press(Message::ToggleContextPage(ContextPage::PageAbout)),
                )
                .push(
                    fl!("kill")
                        .apply(widget::button::destructive)
                        .on_press(Message::ProcessPage(ProcessMessage::KillProcess(
                            self.process_model
                                .item(self.process_model.active())
                                .unwrap()
                                .process
                                .pid,
                        ))),
                )
                .push(
                    fl!("term")
                        .apply(widget::button::suggested)
                        .on_press(Message::ProcessPage(ProcessMessage::TermProcess(
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
                        widget::text::caption(
                            process.cmdline.clone().unwrap_or_default().to_string(),
                        ),
                    ))
                    .add(widget::settings::item(
                        fl!("status"),
                        widget::text::caption(process.state.clone()),
                    ))
                    .apply(Element::from),
                Message::ToggleContextPage(ContextPage::PageAbout),
            ))
        } else {
            None
        }
    }

    fn subscription(&self) -> Vec<Subscription<Message>> {
        vec![Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                use monitord_protocols::protocols::MonitordServiceClient;
                let mut client = MonitordServiceClient::connect("http://127.0.0.1:50051")
                    .await
                    .unwrap();

                let request =
                    tonic::Request::new(monitord_protocols::monitord::ProcessInfoRequest {
                        interval_ms: 1000,
                        username_filter: None,
                        pid_filter: None,
                        name_filter: None,
                        sort_by_cpu: true,
                        sort_by_memory: false,
                        limit: 10000000,
                    });

                let mut response = client
                    .stream_process_info(request)
                    .await
                    .unwrap()
                    .into_inner();

                loop {
                    let message = response.message().await.unwrap();

                    if let Some(item) = message {
                        sender
                            .send(Message::ProcessPage(ProcessMessage::ProcessList(item)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
