use crate::app::message::Message;
use crate::pages::Page;
use cosmic::{widget, Element, Task};

use cosmic::cctk::cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_handle_v1::ZcosmicToplevelHandleV1;
use cosmic::cctk::sctk::output::{OutputHandler, OutputState};
use cosmic::cctk::sctk::registry::{ProvidesRegistryState, RegistryState};
use cosmic::cctk::toplevel_info::{ToplevelInfoHandler, ToplevelInfoState};
use cosmic::cctk::wayland_client::globals::registry_queue_init;
use cosmic::cctk::wayland_client::{protocol::wl_output, Connection, EventQueue, QueueHandle};
use cosmic::cctk::{delegate_toplevel_info, sctk};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::Length;
use cosmic::iced_widget::horizontal_rule;
use sysinfo::System;

pub struct Applications {
    output_state: OutputState,
    registry_state: RegistryState,
    toplevel_info_state: ToplevelInfoState,
}

pub struct ApplicationPage {
    app_data: Applications,
    event_queue: EventQueue<Applications>,
    desktop_entries: Vec<cosmic::desktop::DesktopEntryData>,
}

impl Page for ApplicationPage {
    fn update(&mut self, _sys: &System, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                self.event_queue.roundtrip(&mut self.app_data).unwrap();
            }
            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();
        let header = widget::container(widget::text::heading("Applications"))
            .padding([0, cosmic.space_xs()]);
        let mut col = widget::column();
        for app in self.app_data.toplevel_info_state.toplevels() {
            if let Some(toplevel) = app.1 {
                let mut name = toplevel.app_id.clone();
                let mut icon = cosmic::desktop::IconSource::default();
                for entry in self.desktop_entries.iter() {
                    if entry.id.contains(name.as_str()) {
                        name = entry.name.clone();
                        icon = entry.icon.clone();
                    } else if let Some(wm_class) = entry.wm_class.clone() {
                        if wm_class == name {
                            name = entry.name.clone();
                            icon = entry.icon.clone();
                        }
                    }
                }
                col = col.push(widget::container(
                    widget::row::with_children(vec![
                        icon.as_cosmic_icon().size(24).into(),
                        widget::text::body(name).into(),
                    ])
                    .padding([cosmic.space_xxxs(), cosmic.space_xs()])
                    .spacing(cosmic.space_xs())
                    .width(Length::Fill),
                ))
            }
        }
        widget::container(
            widget::column::with_children(vec![
                header.align_x(Horizontal::Center).into(),
                horizontal_rule(1).into(),
                widget::scrollable(col.spacing(cosmic.space_xxs())).into(),
            ])
            .spacing(cosmic.space_s()),
        )
        .class(cosmic::style::Container::List)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([cosmic.space_m(), cosmic.space_s()])
        .into()
    }
}

impl ApplicationPage {
    pub fn new() -> ApplicationPage {
        let conn = Connection::connect_to_env().expect("Failed to connect to Wayland compositor!");
        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh = event_queue.handle();
        let registry_state = RegistryState::new(&globals);

        ApplicationPage {
            app_data: Applications {
                output_state: OutputState::new(&globals, &qh),
                toplevel_info_state: ToplevelInfoState::new(&registry_state, &qh),
                registry_state,
            },
            event_queue,
            desktop_entries: cosmic::desktop::load_applications(None, false),
        }
    }
}

impl ToplevelInfoHandler for Applications {
    fn toplevel_info_state(&mut self) -> &mut ToplevelInfoState {
        &mut self.toplevel_info_state
    }

    fn new_toplevel(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &ZcosmicToplevelHandleV1) {
    }

    fn update_toplevel(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &ZcosmicToplevelHandleV1,
    ) {
    }

    fn toplevel_closed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &ZcosmicToplevelHandleV1,
    ) {
    }
}

// Need to bind output globals just so toplevel can get output events
impl OutputHandler for Applications {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _: wl_output::WlOutput) {}

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: wl_output::WlOutput,
    ) {
    }
}
sctk::delegate_output!(Applications);

impl ProvidesRegistryState for Applications {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    sctk::registry_handlers!(OutputState);
}
sctk::delegate_registry!(Applications);

delegate_toplevel_info!(Applications);
