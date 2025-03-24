use std::borrow::Cow;

use crate::{app::Message, config::Config, fl, helpers::get_bytes};
use cosmic::{
    app::Task,
    iced::{stream, Subscription},
    prelude::*,
};
use futures_util::SinkExt;
use lazy_static::lazy_static;

use super::ResourceMessage;

lazy_static! {
    static ref NOT_LOADED: Cow<'static, str> = fl!("not-loaded").into();
    static ref MEM_USAGE: Cow<'static, str> = fl!("mem-usage").into();
    static ref SWP_USAGE: Cow<'static, str> = fl!("swp-usage").into();
    static ref MEM_CAP: Cow<'static, str> = fl!("mem-cap").into();
    static ref SWP_CAP: Cow<'static, str> = fl!("swp-cap").into();
}

pub struct MemoryPage {
    device: super::DeviceResource,

    config: Config,
}

impl MemoryPage {
    pub fn new(config: Config) -> Self {
        let mut device = super::DeviceResource::new("");
        device.add_graph(
            MEM_USAGE.clone().into_owned(),
            crate::widget::graph::LineGraph {
                points: vec![0.0; 30],
            },
        );
        device.activate_graph(0);
        Self { device, config }
    }
}

impl super::super::Page for MemoryPage {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::ResourcePage(ResourceMessage::MemorySnapshot(snapshot)) => {
                self.device
                    .add_info(MEM_CAP.clone(), get_bytes(snapshot.total_memory_bytes));
                self.device
                    .add_info(SWP_CAP.clone(), get_bytes(snapshot.swap_total_bytes));
                self.device
                    .set_statistic(MEM_USAGE.clone(), get_bytes(snapshot.used_memory_bytes));
                self.device
                    .set_statistic(SWP_USAGE.clone(), get_bytes(snapshot.swap_used_bytes));
                self.device.push_graph(
                    MEM_USAGE.clone(),
                    ((snapshot.used_memory_bytes + snapshot.swap_used_bytes) as f64
                        / (snapshot.total_memory_bytes + snapshot.swap_total_bytes) as f64)
                        as f32,
                );
            }
            Message::UpdateConfig(config) => self.config = config,

            Message::ResourcePage(ResourceMessage::SelectDeviceTab(tab)) => {
                if self.device.contains_tab(tab) {
                    self.device.activate_tab(tab)
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        self.device.view()
    }

    fn subscription(&self) -> Vec<Subscription<Message>> {
        vec![Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                use monitord_protocols::protocols::MonitordServiceClient;
                let mut client = MonitordServiceClient::connect("http://127.0.0.1:50051")
                    .await
                    .unwrap();

                let request = tonic::Request::new(monitord_protocols::monitord::SnapshotRequest {
                    interval_ms: 1000,
                });

                let mut stream = client
                    .stream_memory_info(request)
                    .await
                    .unwrap()
                    .into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(item) = message {
                        sender
                            .send(Message::ResourcePage(ResourceMessage::MemorySnapshot(item)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
