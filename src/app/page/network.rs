use std::collections::{HashMap, VecDeque};

use cosmic::{
    iced::{self, stream, Subscription},
    prelude::*,
    widget,
};
use futures_util::SinkExt;
use monitord_protocols::{
    monitord::{NetworkInfo, NetworkList, SnapshotRequest},
    protocols::MonitordServiceClient,
};

use crate::{app::Message, fl};

#[derive(Debug, Clone)]
pub enum NetworkMessage {
    Snapshot(NetworkList),
    SelectTab(widget::segmented_button::Entity),
}

struct NetworkDevice {
    info: NetworkInfo,
    history: VecDeque<f32>,
}

pub struct NetworkPage {
    net_list: widget::segmented_button::SingleSelectModel,
    name_to_entity: HashMap<String, widget::segmented_button::Entity>,
}

impl NetworkPage {
    pub fn new() -> Self {
        Self {
            net_list: widget::segmented_button::SingleSelectModel::default(),
            name_to_entity: HashMap::new(),
        }
    }
}

impl super::Page for NetworkPage {
    fn update(&mut self, msg: Message) -> cosmic::app::Task<Message> {
        let tasks = Vec::new();

        match msg {
            Message::NetworkPage(NetworkMessage::Snapshot(snapshot)) => {
                for net in snapshot.nets.iter() {
                    let entity = if let Some(entity) = self.name_to_entity.get(&net.interface_name)
                    {
                        entity.clone()
                    } else {
                        let entity = self
                            .net_list
                            .insert()
                            .text(net.interface_name.clone())
                            .data(NetworkDevice {
                                info: net.clone(),
                                history: VecDeque::from(vec![0.0; 30]),
                            })
                            .id();
                        self.name_to_entity
                            .insert(net.interface_name.clone(), entity.clone());
                        entity
                    };
                    let device = self.net_list.data_mut::<NetworkDevice>(entity).unwrap();
                    device.info = net.clone();
                    device
                        .history
                        .push_back(net.rx_bytes_per_sec as f32 + net.tx_bytes_per_sec as f32);
                    device.history.pop_front();
                }
            }
            Message::NetworkPage(NetworkMessage::SelectTab(tab)) => self.net_list.activate(tab),
            _ => {}
        }

        cosmic::app::Task::batch(tasks)
    }

    fn view(&self) -> cosmic::Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();

        widget::column()
            .spacing(cosmic.space_xs())
            .push(
                widget::tab_bar::horizontal(&self.net_list)
                    .on_activate(|entity| Message::NetworkPage(NetworkMessage::SelectTab(entity))),
            )
            .push_maybe(self.net_list.active_data::<NetworkDevice>().map(|net| {
                widget::row()
                    .spacing(cosmic.space_xxs())
                    .push(
                        widget::canvas(crate::widget::graph::LineGraph {
                            points: {
                                let max = net
                                    .history
                                    .iter()
                                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                                    .unwrap()
                                    .max(1.0);
                                net.history.iter().cloned().map(|val| val / max).collect()
                            },
                        })
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill),
                    )
                    .push(
                        widget::settings::view_column(vec![
                            widget::settings::section()
                                .title(fl!("net-info"))
                                .add(widget::settings::item(
                                    fl!("interface-name"),
                                    net.info.interface_name.clone().apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("net-driver"),
                                    net.info.driver.clone().apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("mac-address"),
                                    net.info.mac_address.clone().apply(widget::text::body),
                                ))
                                .apply(Element::from),
                            widget::settings::section()
                                .title(fl!("net-stats"))
                                .add(widget::settings::item(
                                    fl!("bandwidth"),
                                    format!(
                                        "{}/s",
                                        net.info
                                            .max_bandwidth_bytes_per_sec
                                            .apply(crate::helpers::get_bytes)
                                    )
                                    .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("rx-bytes"),
                                    format!(
                                        "{}/s",
                                        net.info.rx_bytes_per_sec.apply(crate::helpers::get_bytes)
                                    )
                                    .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("rx-packets"),
                                    format!("{}/s", net.info.rx_packets_per_sec)
                                        .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("rx-total"),
                                    net.info
                                        .rx_bytes_total
                                        .apply(crate::helpers::get_bytes)
                                        .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("rx-errors"),
                                    format!("{}/s", net.info.rx_errors).apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("tx-bytes"),
                                    format!(
                                        "{}/s",
                                        net.info.tx_bytes_per_sec.apply(crate::helpers::get_bytes)
                                    )
                                    .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("tx-packets"),
                                    format!("{}/s", net.info.tx_packets_per_sec)
                                        .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("tx-total"),
                                    net.info
                                        .tx_bytes_total
                                        .apply(crate::helpers::get_bytes)
                                        .apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("tx-errors"),
                                    format!("{}/s", net.info.tx_errors).apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("is-up"),
                                    net.info.is_up.to_string().apply(widget::text::body),
                                ))
                                .add(widget::settings::item(
                                    fl!("mtu"),
                                    net.info.mtu.to_string().apply(widget::text::body),
                                ))
                                .apply(Element::from),
                        ])
                        .apply(widget::scrollable),
                    )
                    .apply(Element::from)
            }))
            .apply(Element::from)
    }

    fn subscription(&self) -> Vec<Subscription<Message>> {
        vec![Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let mut service = MonitordServiceClient::connect("http://127.0.0.1:50051")
                    .await
                    .unwrap();

                let request = tonic::Request::new(SnapshotRequest { interval_ms: 1000 });

                let mut stream = service
                    .stream_network_info(request)
                    .await
                    .unwrap()
                    .into_inner();

                loop {
                    let message = stream.message().await.unwrap();

                    if let Some(message) = message {
                        sender
                            .send(Message::NetworkPage(NetworkMessage::Snapshot(message)))
                            .await
                            .unwrap();
                    }
                }
            })
        })]
    }
}
