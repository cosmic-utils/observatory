use crate::{app::Message, fl};
use cosmic::{
    iced::{stream, Subscription},
    prelude::*,
    widget,
};
use futures_util::SinkExt;
use monitord_protocols::{
    monitord::{SnapshotRequest, SystemInfo},
    protocols::MonitordServiceClient,
};

/// Messages that are emitted that are relevant to the System page
#[derive(Debug, Clone)]
pub enum SystemMessage {
    Snapshot(SystemInfo),
}

pub struct SystemPage {
    system_info: Option<SystemInfo>,
}

impl SystemPage {
    pub fn new() -> Self {
        Self { system_info: None }
    }
}

impl super::Page for SystemPage {
    fn update(&mut self, msg: crate::app::Message) -> cosmic::app::Task<crate::app::Message> {
        let tasks = Vec::new();
        match msg {
            Message::SystemPage(SystemMessage::Snapshot(snapshot)) => {
                self.system_info = Some(snapshot);
            }
            _ => {}
        }

        cosmic::app::Task::batch(tasks)
    }

    fn view(&self) -> Element<Message> {
        if let Some(system_info) = &self.system_info {
            widget::settings::view_column(vec![
                widget::settings::section()
                    .title(fl!("os-info"))
                    .add(widget::settings::item(
                        fl!("hostname"),
                        system_info.hostname.clone().apply(widget::text::body),
                    ))
                    .add(widget::settings::item(
                        fl!("os-name"),
                        system_info.os_name.clone().apply(widget::text::body),
                    ))
                    .add(widget::settings::item(
                        fl!("os-version"),
                        system_info.os_version.clone().apply(widget::text::body),
                    ))
                    .add(widget::settings::item(
                        fl!("kernel-version"),
                        system_info.kernel_version.clone().apply(widget::text::body),
                    ))
                    .add_maybe(system_info.vendor.clone().map(|vendor| {
                        widget::settings::item(fl!("vendor"), vendor.apply(widget::text::body))
                    }))
                    .apply(Element::from),
                widget::settings::section()
                    .title(fl!("system-stats"))
                    .add(widget::settings::item(
                        fl!("process-count"),
                        system_info
                            .process_count
                            .to_string()
                            .apply(widget::text::body),
                    ))
                    .add(widget::settings::item(
                        fl!("thread-count"),
                        system_info
                            .thread_count
                            .to_string()
                            .apply(widget::text::body),
                    ))
                    .add(widget::settings::item(
                        fl!("open-files"),
                        system_info
                            .open_file_count
                            .to_string()
                            .apply(widget::text::body),
                    ))
                    .apply(Element::from),
            ])
            .apply(widget::scrollable)
            .apply(Element::from)
        } else {
            widget::horizontal_space().apply(Element::from)
        }
    }

    fn subscription(&self) -> Vec<Subscription<Message>> {
        vec![Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let mut service = MonitordServiceClient::connect("http://127.0.0.1:50051")
                    .await
                    .unwrap();

                let request = tonic::Request::new(SnapshotRequest { interval_ms: 1000 });

                let mut stream = service
                    .stream_system_info(request)
                    .await
                    .unwrap()
                    .into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(message) = message {
                        sender
                            .send(Message::SystemPage(SystemMessage::Snapshot(message)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
