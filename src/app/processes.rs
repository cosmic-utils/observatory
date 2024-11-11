use crate::app::message::Message;
use crate::app::cosmic_theming;

use cosmic::iced::Alignment;
use cosmic::widget::horizontal_space;
use cosmic::{
    iced::Length,
    iced_widget::{horizontal_rule, row, scrollable},
    widget,
    widget::{button, container, icon, text},
    Element,
};
use cosmic::iced::alignment::Horizontal;

#[derive(Clone, Debug)]
pub struct Process {
    name: String,
    user: String,
    cpu: String,
    mem: String,
    disk: String,
    pid: sysinfo::Pid,
}

pub struct ProcessPage {
    sort_data: (HeaderCategory, SortDirection),
    users: sysinfo::Users,
    active_uid: sysinfo::Uid,

    processes: Vec<Process>,

    selected_process: Option<sysinfo::Pid>,
}

impl ProcessPage {
    pub fn new(sys: &sysinfo::System) -> ProcessPage {
        ProcessPage {
            processes: vec![],
            sort_data: (HeaderCategory::Name, SortDirection::Descending),
            users: sysinfo::Users::new_with_refreshed_list(),
            active_uid: sys
                .processes()
                .values()
                .find(|process| process.name() == "cosmic-monitor")
                .unwrap()
                .user_id()
                .unwrap()
                .clone(),
            selected_process: None,
        }
    }

    pub fn update(&mut self, sys: &sysinfo::System, message: Message) {
        match message {
            Message::ProcessTermActive => {
                sys.process(self.selected_process.unwrap())
                    .unwrap()
                    .kill_with(sysinfo::Signal::Term)
                    .unwrap();
            }
            Message::ProcessKillActive => {
                sys.process(self.selected_process.unwrap()).unwrap().kill();
            }
            Message::ProcessClick(pid) => {
                self.selected_process = pid
            }
            Message::Refresh => {
                self.update_processes(sys);
            }
        };
    }

    pub fn view(&self) -> Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();

        // The vertical column of process elements
        let mut main_column = cosmic::widget::column::<Message>().height(Length::Fill);

        // Header row
        main_column = main_column
            .push(self.create_header_row(&theme))
            .push(container(horizontal_rule(1)).padding([cosmic.space_xxs(), 0]));

        // Push process rows into scrollable widget
        let mut process_group = widget::column().spacing(cosmic.space_xxxs()).padding([0, cosmic.space_xxs(), 0, 0]);
        for process in &self.processes {
            process_group = process_group.push(self.create_process_row(&theme, &process));
        }
        let process_group_scroll = scrollable(
            widget::mouse_area(process_group).on_press(Message::ProcessClick(None)),
        ).width(Length::Fill);

        main_column.push(process_group_scroll).into()
    }

    pub fn footer(&self) -> Element<Message> {
        if self.selected_process.is_some() {
            let mut col = widget::column::with_capacity::<Message>(1);
            let theme = cosmic::theme::active();
            let cosmic = theme.cosmic();

            let mut row = widget::row::with_capacity(2)
                .align_y(Alignment::Center)
                .spacing(cosmic.space_xxs());
            row = row.push(horizontal_space());
            row = row.push(container(
                button::standard("Kill").on_press(Message::ProcessKillActive),
            ));
            row = row.push(container(
                button::suggested("Terminate").on_press(Message::ProcessTermActive),
            ));

            col = col.push(row);

            widget::layer_container(col)
                .layer(cosmic::cosmic_theme::Layer::Primary)
                .padding([cosmic.space_xxs(), cosmic.space_xs()])
                .into()
        } else {
            widget::row().into()
        }
    }

    fn get_process_name(process: &sysinfo::Process) -> String {
        // Check if the cmd file name starts with process.name()
        let name = process.name().to_str().unwrap();
        let cmd = if process.cmd().len() > 0 { process.cmd()[0].to_str() } else { None };

        if let Some(cmd) = cmd {
            let file_name = std::path::Path::new(cmd).file_name();
            if let Some(file_name) = file_name {
                // Now that we've established the cmd, let's check that name starts with it!
                if file_name.to_str().unwrap().starts_with(name) {
                    return file_name.to_str().unwrap().to_owned();
                }
            }
        }
        name.into()
    }

    pub fn update_processes(&mut self, sys: &sysinfo::System) {
        self.processes = sys
            .processes()
            .values()
            .filter(|process| process.thread_kind().is_none() && process.user_id() == Some(&self.active_uid))
            .map(|process| Process {
                name: Self::get_process_name(process),
                user: self
                    .users
                    .get_user_by_id(process.user_id().unwrap())
                    .unwrap()
                    .name()
                    .into(),
                cpu: format!("{:.1}%", process.cpu_usage() / sys.cpus().len() as f32),
                mem: {
                    let bytes = process.memory();
                    let kibibytes = bytes as f64 / 1024.;
                    let mebibytes = kibibytes / 1024.;
                    let gibibytes = mebibytes / 1024.;
                    if bytes < 1024 {
                        format!("{} B", bytes)
                    } else if kibibytes < 1024. {
                        format!("{:.2} KiB", kibibytes)
                    } else if mebibytes < 1024. {
                        format!("{:.2} MiB", mebibytes)
                    } else {
                        format!("{:.2} GiB", gibibytes)
                    }
                },
                disk: {
                    let bytes =
                        process.disk_usage().read_bytes + process.disk_usage().written_bytes;
                    let kibibytes = bytes as f64 / 1024.;
                    let mebibytes = kibibytes / 1024.;
                    let gibibytes = mebibytes / 1024.;
                    if bytes < 1024 {
                        format!("{} B/s", bytes)
                    } else if kibibytes < 1024. {
                        format!("{:.2} KiB/s", kibibytes)
                    } else if mebibytes < 1024. {
                        format!("{:.2} MiB/s", mebibytes)
                    } else {
                        format!("{:.2} GiB/s", gibibytes)
                    }
                },
                pid: process.pid(),
            })
            .collect();

        self.sort_processes();
    }

    fn sort_processes(&mut self) {
        self.processes.sort_by(|a, b| match self.sort_data {
            (HeaderCategory::Name, SortDirection::Descending) => a
                .name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase()),
            (HeaderCategory::Name, SortDirection::_Ascending) => b
                .name
                .to_ascii_lowercase()
                .cmp(&a.name.to_ascii_lowercase()),
            _ => std::cmp::Ordering::Less,
        })
    }

    fn create_header_row(&self, theme: &cosmic::theme::Theme) -> Element<Message> {
        let cosmic = theme.cosmic();
        let mut row = widget::row::with_capacity::<Message>(5)
            .spacing(cosmic.space_xxxs())
            .padding([0, cosmic.space_xxs()]);

        let sort_arrow = |category: HeaderCategory| -> widget::Container<Message, cosmic::theme::Theme> {
            if self.sort_data.0 == category {
                if self.sort_data.1 == SortDirection::Descending {
                    container(icon::from_name("pan-down-symbolic"))
                } else {
                    container(icon::from_name("pan-up-symbolic"))
                }
            } else {
                container(text::body(""))
            }
        };

        row = row.push(
            container(row![text::heading("Name"), sort_arrow(HeaderCategory::Name)].spacing(cosmic.space_xxxs()))
                .width(HeaderCategory::width(&HeaderCategory::Name))
        );
        row = row.push(
            container(row![text::heading("User"), sort_arrow(HeaderCategory::User)].spacing(cosmic.space_xxxs()))
                .width(HeaderCategory::width(&HeaderCategory::User))
        );
        row = row.push(
            container(row![text::heading("CPU"), sort_arrow(HeaderCategory::Cpu)].spacing(cosmic.space_xxxs()))
                .width(HeaderCategory::width(&HeaderCategory::Cpu))
                .align_x(Horizontal::Right)
        );
        row = row.push(
            container(row![text::heading("Memory"), sort_arrow(HeaderCategory::Memory)].spacing(cosmic.space_xxxs()))
                .width(HeaderCategory::width(&HeaderCategory::Memory))
                .align_x(Horizontal::Right)
        );
        row = row.push(
            container(row![text::heading("Disk"), sort_arrow(HeaderCategory::Disk)].spacing(cosmic.space_xxxs()))
                .width(HeaderCategory::width(&HeaderCategory::Disk))
                .align_x(Horizontal::Right)
        );

        Element::new(row)
    }

    fn create_process_row<'a>(&'a self, theme: &cosmic::Theme, process: &'a Process) -> Element<'a, Message> {
        let cosmic = theme.cosmic();
        let mut row = widget::row::with_capacity::<Message>(5)
            .spacing(cosmic.space_xxxs())
            .padding([0, cosmic.space_xxs()]);

        row = row.push(
            widget::container(text::body(&process.name))
                .width(HeaderCategory::width(&HeaderCategory::Name)));
        row = row.push(
            widget::container(text::body(&process.user))
                .width(HeaderCategory::width(&HeaderCategory::User)));
        row = row.push(
            widget::container(text::body(&process.cpu))
                .align_x(Horizontal::Right)
                .width(HeaderCategory::width(&HeaderCategory::Cpu)));
        row = row.push(
            widget::container(text::body(&process.mem))
                .align_x(Horizontal::Right)
                .width(HeaderCategory::width(&HeaderCategory::Memory)));
        row = row.push(
            widget::container(text::body(&process.disk))
                .align_x(Horizontal::Right)
                .width(HeaderCategory::width(&HeaderCategory::Disk)));


        button::custom(row)
            .padding([cosmic.space_xxxs(), 0])
            .width(Length::Fill)
            .class(cosmic_theming::button_style(
                match self.selected_process {
                    Some(active_process) => active_process == process.pid,
                    _ => false
                },
                true,
            ))
            .on_press(Message::ProcessClick(Some(process.pid)))
            .width(Length::Shrink)
            .into()
    }
}

#[derive(PartialEq)]
enum SortDirection {
    _Ascending,
    Descending,
}

#[derive(PartialEq, PartialOrd)]
enum HeaderCategory {
    Name,
    User,
    Cpu,
    Memory,
    Disk,
}

impl HeaderCategory {
    // (300, 100, 75, 100, 150);
    fn width(cat: &HeaderCategory) -> Length {
        match cat {
            HeaderCategory::Name => 300.into(),
            HeaderCategory::User => 80.into(),
            HeaderCategory::Cpu => 60.into(),
            HeaderCategory::Memory => 80.into(),
            HeaderCategory::Disk => 100.into(),
        }
    }
}
