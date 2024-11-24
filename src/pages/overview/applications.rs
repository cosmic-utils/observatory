use crate::app::message::Message;
use crate::pages::Page;
use cosmic::{widget, Element, Task};

use cosmic::cctk::cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_handle_v1::ZcosmicToplevelHandleV1;
use cosmic::cctk::cosmic_protocols::toplevel_management::v1::client::zcosmic_toplevel_manager_v1::ZcosmicToplelevelManagementCapabilitiesV1;
use cosmic::cctk::sctk::output::{OutputHandler, OutputState};
use cosmic::cctk::sctk::registry::{ProvidesRegistryState, RegistryState};
use cosmic::cctk::toplevel_info::{ToplevelInfo, ToplevelInfoHandler, ToplevelInfoState};
use cosmic::cctk::toplevel_management::{ToplevelManagerHandler, ToplevelManagerState};
use cosmic::cctk::wayland_client::globals::registry_queue_init;
use cosmic::cctk::wayland_client::{
    protocol::wl_output, Connection, EventQueue, QueueHandle, WEnum,
};
use cosmic::cctk::{delegate_toplevel_info, sctk};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Background, Length};
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
    active_toplevel: Option<ToplevelInfo>,
}

impl Page for ApplicationPage {
    fn update(&mut self, _sys: &System, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                self.event_queue.roundtrip(&mut self.app_data).unwrap();
            }
            Message::ApplicationSelect(app_id) => {
                if let Some(toplevel) = self
                    .app_data
                    .toplevel_info_state
                    .toplevels()
                    .find(|toplevel| toplevel.1.unwrap().app_id == app_id)
                {
                    self.active_toplevel = toplevel.1.cloned();
                }
            }
            Message::ApplicationClose => {
                if let Some(active_toplevel) = self.active_toplevel.take() {
                    if let Some(toplevel) = self
                        .app_data
                        .toplevel_info_state
                        .toplevels()
                        .find(|toplevel| toplevel.1.unwrap().app_id == active_toplevel.app_id)
                        .take()
                    {
                        // TODO
                    }
                }
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
                let is_selected = self
                    .active_toplevel
                    .clone()
                    .is_some_and(|tl| tl.app_id == toplevel.app_id);
                col = col.push(
                    widget::button::custom(
                        widget::row::with_children(vec![
                            icon.as_cosmic_icon().size(24).into(),
                            widget::text::body(name.clone()).into(),
                        ])
                        .align_y(Vertical::Center)
                        .padding([cosmic.space_xxxs(), cosmic.space_xs()])
                        .spacing(cosmic.space_xs())
                        .width(Length::Fill),
                    )
                    .on_press(Message::ApplicationSelect(toplevel.app_id.clone()))
                    .class(cosmic::style::Button::Custom {
                        active: Box::new(move |_, theme| {
                            let cosmic = theme.cosmic();
                            let mut appearance = widget::button::Style::new();
                            if is_selected {
                                appearance.background =
                                    Some(Background::Color(cosmic.accent.base.into()));
                                appearance.text_color = Some(cosmic.accent.on.into());
                            }
                            appearance.border_radius = cosmic.radius_s().into();
                            appearance
                        }),

                        disabled: Box::new(move |theme| {
                            let cosmic = theme.cosmic();
                            let mut appearance = widget::button::Style::new();
                            if is_selected {
                                appearance.background =
                                    Some(Background::Color(cosmic.accent.disabled.into()));
                                appearance.text_color = Some(cosmic.accent.on.into());
                            } else {
                                appearance.background =
                                    Some(Background::Color(cosmic.button.disabled.into()));
                                appearance.text_color = Some(cosmic.button.on_disabled.into());
                            }

                            appearance
                        }),
                        hovered: Box::new(move |_, theme| {
                            let cosmic = theme.cosmic();
                            let mut appearance = widget::button::Style::new();
                            if is_selected {
                                appearance.background =
                                    Some(Background::Color(cosmic.accent.hover.into()));
                                appearance.text_color = Some(cosmic.accent.on.into());
                            } else {
                                appearance.background =
                                    Some(Background::Color(cosmic.button.hover.into()));
                                appearance.text_color = Some(cosmic.button.on.into());
                            }
                            appearance.border_radius = cosmic.radius_s().into();
                            appearance
                        }),
                        pressed: Box::new(move |_, theme| {
                            let cosmic = theme.cosmic();
                            let mut appearance = widget::button::Style::new();
                            if is_selected {
                                appearance.background =
                                    Some(Background::Color(cosmic.accent.pressed.into()));
                                appearance.text_color = Some(cosmic.accent.on.into());
                            } else {
                                appearance.background =
                                    Some(Background::Color(cosmic.button.pressed.into()));
                                appearance.text_color = Some(cosmic.button.on.into());
                            }
                            appearance.border_radius = cosmic.radius_s().into();
                            appearance
                        }),
                    }),
                );
                log::warn!("Selected color: {:?}", cosmic.text_button.selected);
            }
        }
        widget::container(
            widget::column::with_children(vec![
                header.align_x(Horizontal::Center).into(),
                horizontal_rule(1).into(),
                widget::scrollable(col.spacing(cosmic.space_xxxs())).into(),
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
            active_toplevel: None,
        }
    }
}

impl ToplevelManagerHandler for ApplicationPage {
    fn toplevel_manager_state(&mut self) -> &mut ToplevelManagerState {
        todo!()
    }

    fn capabilities(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        capabilities: Vec<WEnum<ZcosmicToplelevelManagementCapabilitiesV1>>,
    ) {
        todo!()
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
