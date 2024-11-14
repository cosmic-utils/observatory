use cosmic::{
    iced::{Length, alignment::{Vertical, Horizontal}},
    widget,
    Element,
};
use cosmic::iced;
use cosmic::iced_widget;

use crate::app::applications::Application;
use crate::app::message::Message;
use crate::cosmic_theming;

pub struct ProcessPage {
    sort_data: (HeaderCategory, SortDirection),
    users: sysinfo::Users,
    active_uid: sysinfo::Uid,
    categories: Vec<HeaderCategory>,
    processes: Vec<Process>,
    selected_process: Option<sysinfo::Pid>,

}

impl ProcessPage {
    pub fn new(sys: &sysinfo::System) -> Self {
        Self {
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
            categories: vec![HeaderCategory::Name, HeaderCategory::User, HeaderCategory::Cpu, HeaderCategory::Memory, HeaderCategory::Disk],
            processes: vec![],
            selected_process: None,
        }
    }

    pub fn update(&mut self, sys: &sysinfo::System, message: Message, apps: &Vec<Application>) {
        match message {
            Message::ProcessTermActive => {
                sys.process(self.selected_process.unwrap())
                    .unwrap()
                    .kill_with(sysinfo::Signal::Term)
                    .unwrap();
                self.selected_process = None;
            }
            Message::ProcessKillActive => {
                sys.process(self.selected_process.unwrap()).unwrap().kill();
                self.selected_process = None;
            }
            Message::ProcessClick(pid) => {
                self.selected_process = pid
            }
            Message::ProcessCategoryClick(index) => {
                let cat = HeaderCategory::from_index(index).unwrap();
                if cat == self.sort_data.0 {
                    self.sort_data.1.opposite();
                } else {
                    self.sort_data = (cat, SortDirection::Descending);
                }
                self.sort_processes();
            }
            Message::Refresh => {
                self.update_processes(sys, apps);
            }
            Message::KeyPressed(key) => {
                if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                    self.selected_process = None;
                }
            }
            _ => {}
        };
    }

    pub fn view(&self) -> Element<Message> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();

        // The vertical column of process elements
        let mut main_column = cosmic::widget::column::<Message>().height(Length::Fill)
            .width(Length::Fixed(1024.));

        // Header row
        main_column = main_column
            .push(widget::row::with_children(
                self.categories.iter()
                    .map(|cat| cat.element(&theme, &self.sort_data))
                    .collect())
                .spacing(cosmic.space_xxxs())
                .padding([cosmic.space_xxs(), cosmic.space_xxs()]))
            .push(iced_widget::horizontal_rule(1));

        let mut process_column = widget::column().spacing(cosmic.space_xxxs()).padding([0, cosmic.space_xxs(), 0, 0]);
        for process in &self.processes {
            process_column = process_column.push(process.element(&self.categories, &theme, self.selected_process == Some(process.pid)));
        }
        // Push process rows into scrollable widget
        let process_group_scroll = widget::context_menu(
            iced_widget::Scrollable::with_direction(
                process_column,
                iced_widget::scrollable::Direction::Both {
                    horizontal: iced_widget::scrollable::Scrollbar::default(),
                    vertical: iced_widget::scrollable::Scrollbar::default(),
                }).width(Length::Fill),
            ContextMenuAction::menu());

        main_column.push(process_group_scroll).into()
    }

    pub fn footer(&self) -> Option<Element<Message>> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();

        let mut row = widget::row::with_capacity(4)
            .align_y(Vertical::Center)
            .spacing(cosmic.space_xs());
        row = row.push(widget::horizontal_space());
        if self.selected_process.is_some() {
            row = row.push(widget::button::destructive("Kill").on_press(Message::ProcessKillActive));
            row = row.push(widget::button::suggested("Terminate").on_press(Message::ProcessTermActive));
        } else {
            row = row.push(widget::button::destructive("Kill"));
            row = row.push(widget::button::suggested("Terminate"));
        }

        Some(widget::layer_container(row)
            .layer(cosmic::cosmic_theme::Layer::Primary)
            .padding([cosmic.space_xxs(), cosmic.space_xs()])
            .into())
    }

    pub fn update_processes(&mut self, sys: &sysinfo::System, apps: &Vec<Application>) {
        self.processes = sys
            .processes()
            .values()
            .filter(|process| process.thread_kind().is_none() && process.user_id() == Some(&self.active_uid))
            .map(|process| Process::from_process(process, apps, sys, &self.users))
            .collect();

        self.sort_processes();
    }

    fn sort_processes(&mut self) {
        self.processes.sort_by(|a, b| match self.sort_data.1 {
            SortDirection::Ascending => Process::compare(a, b, &self.sort_data.0),
            SortDirection::Descending => Process::compare(b, a, &self.sort_data.0),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Process {
    icon: String,
    name: String,
    user: String,
    cpu: String,
    cpu_percent: f32,
    mem: String,
    mem_bytes: u64,
    disk: String,
    disk_bytes: u64,
    pid: sysinfo::Pid,
}


impl Process {
    fn from_process(process: &sysinfo::Process, apps: &Vec<Application>, sys: &sysinfo::System, users: &sysinfo::Users) -> Self {
        Self {
            icon: match apps.iter().find(|app| {
                if let Some(cmd) = process.cmd().iter().nth(0) {
                    app.cmd() == cmd
                } else {
                    false
                }
            }) {
                Some(app) => app.icon(),
                None => "application-default-symbolic"
            }.into(),
            name: Process::get_name(process),
            user: users
                .get_user_by_id(process.user_id().unwrap())
                .unwrap()
                .name()
                .into(),
            cpu: format!("{:.1}%", process.cpu_usage() / sys.cpus().len() as f32),
            cpu_percent: process.cpu_usage() / sys.cpus().len() as f32,
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
            mem_bytes: process.memory(),
            disk: {
                let bytes =
                    process.disk_usage().read_bytes + process.disk_usage().written_bytes;
                let kibibytes = bytes as f64 / 1024.;
                let mebibytes = kibibytes / 1024.;
                let gibibytes = mebibytes / 1024.;
                if bytes < 1024 {
                    format!("{} B/s", bytes)
                } else if kibibytes < 1024. {
                    format!("{:.1} KiB/s", kibibytes)
                } else if mebibytes < 1024. {
                    format!("{:.1} MiB/s", mebibytes)
                } else {
                    format!("{:.1} GiB/s", gibibytes)
                }
            },
            disk_bytes: process.disk_usage().read_bytes + process.disk_usage().written_bytes,
            pid: process.pid(),
        }
    }

    fn compare(a: &Process, b: &Process, cat: &HeaderCategory) -> std::cmp::Ordering {
        match cat {
            HeaderCategory::Name => {
                let mut ord = b.name.to_ascii_lowercase().cmp(&a.name.to_ascii_lowercase());
                if ord == std::cmp::Ordering::Equal {
                    ord = a.pid.cmp(&b.pid);
                }
                ord
            }
            HeaderCategory::User => {
                let mut ord = a.user.cmp(&b.user);
                if ord == std::cmp::Ordering::Equal {
                    ord = Self::compare(a, b, &HeaderCategory::Name);
                }
                ord
            }
            HeaderCategory::Cpu => a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap(),
            HeaderCategory::Memory => {
                a.mem_bytes.partial_cmp(&b.mem_bytes).unwrap()
            }
            HeaderCategory::Disk => {
                let mut ord = a.disk_bytes.partial_cmp(&b.disk_bytes).unwrap();
                if ord == std::cmp::Ordering::Equal {
                    ord = Self::compare(a, b, &HeaderCategory::Name);
                }
                ord
            }
        }
    }

    // Returns two strings, first is the optional icon and second is the actual data
    fn category(&self, category: &HeaderCategory) -> String {
        match category {
            HeaderCategory::Name => self.name.clone(),
            HeaderCategory::User => self.user.clone(),
            HeaderCategory::Cpu => self.cpu.clone(),
            HeaderCategory::Memory => self.mem.clone(),
            HeaderCategory::Disk => self.disk.clone(),
        }
    }

    fn get_name(process: &sysinfo::Process) -> String {
        // Check if the cmd file name starts with process.name()
        let name = process.name().to_str().unwrap();
        let cmd = process.cmd().iter().nth(0);

        if let Some(cmd) = cmd {
            let file_name = std::path::Path::new(cmd).file_name();
            if let Some(file_name) = file_name {
                let file_name = file_name.to_str().unwrap().split(' ').nth(0).unwrap();
                // Now that we've established the cmd, let's check that name starts with it!
                if file_name.starts_with(name) {
                    return file_name.to_owned();
                }
            }
        }
        name.into()
    }

    fn element(&self, categories: &Vec<HeaderCategory>, theme: &cosmic::Theme, is_selected: bool) -> Element<Message> {
        let cosmic = theme.cosmic();
        let row = widget::row::with_children::<Message>(categories.iter().map(|cat|
            self.category_element(theme, cat)
        ).collect())
            .spacing(HeaderCategory::spacing())
            .align_y(Vertical::Center);

        // Create the button widget
        widget::button::custom(row)
            .padding([cosmic.space_xxs(), cosmic.space_xs()])
            .width(Length::Fill)
            .class(cosmic_theming::button_style(is_selected, true))
            .on_press(Message::ProcessClick(Some(self.pid)))
            .width(Length::Shrink)
            .into()
    }

    fn category_element(&self, theme: &cosmic::Theme, category: &HeaderCategory) -> Element<Message> {
        let cosmic = theme.cosmic();
        let mut row = widget::row::with_capacity::<Message>(2)
            .spacing(cosmic.space_xxs())
            .align_y(Vertical::Center);
        // Add the icon for the name
        if category == &HeaderCategory::Name {
            row = row.push(widget::container(widget::icon::from_name(self.icon.as_str()))
                .align_x(Horizontal::Center))
        }

        row = row.push(widget::text::body(self.category(&category)));

        widget::container(row)
            .width(category.width())
            .align_x(category.alignment())
            .into()
    }
}

#[derive(PartialEq)]
enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn opposite(&mut self) {
        if *self == SortDirection::Ascending { *self = SortDirection::Descending; } else { *self = SortDirection::Ascending }
    }
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
    fn name(&self) -> String {
        match self {
            HeaderCategory::Name => "Name",
            HeaderCategory::User => "User",
            HeaderCategory::Cpu => "CPU",
            HeaderCategory::Memory => "Memory",
            HeaderCategory::Disk => "Disk",
        }.into()
    }

    fn alignment(&self) -> Horizontal {
        match self {
            HeaderCategory::Name => Horizontal::Left,
            HeaderCategory::User => Horizontal::Left,
            HeaderCategory::Cpu => Horizontal::Right,
            HeaderCategory::Memory => Horizontal::Right,
            HeaderCategory::Disk => Horizontal::Right,
        }
    }

    fn spacing() -> u16 {
        cosmic::theme::active().cosmic().space_xs()
    }

    fn width(&self) -> Length {
        match self {
            HeaderCategory::Name => 300,
            HeaderCategory::User => 80,
            HeaderCategory::Cpu => 60,
            HeaderCategory::Memory => 80,
            HeaderCategory::Disk => 100,
        }.into()
    }

    fn index(&self) -> u8 {
        match self {
            HeaderCategory::Name => 0,
            HeaderCategory::User => 1,
            HeaderCategory::Cpu => 2,
            HeaderCategory::Memory => 3,
            HeaderCategory::Disk => 4,
        }
    }

    fn from_index(index: u8) -> Option<HeaderCategory> {
        match index {
            0 => Some(HeaderCategory::Name),
            1 => Some(HeaderCategory::User),
            2 => Some(HeaderCategory::Cpu),
            3 => Some(HeaderCategory::Memory),
            4 => Some(HeaderCategory::Disk),
            _ => None,
        }
    }

    fn element(&self, theme: &cosmic::theme::Theme, sort_data: &(HeaderCategory, SortDirection)) -> Element<Message> {
        let cosmic = theme.cosmic();
        let mut row = widget::row::with_capacity::<Message>(2)
            .align_y(Vertical::Center)
            .spacing(cosmic.space_xxs());
        row = row.push(widget::text::heading(self.name()));
        if &sort_data.0 == self {
            match sort_data.1 {
                SortDirection::Descending => row = row.push(widget::container(widget::icon::from_name("pan-down-symbolic"))),
                SortDirection::Ascending => row = row.push(widget::container(widget::icon::from_name("pan-up-symbolic"))),
            }
        }

        // Button for changing the sort direction

        widget::mouse_area(widget::container(row)
            .width(self.width())
        )
            .on_press(Message::ProcessCategoryClick(self.index()))
            .into()
    }
}

#[derive(PartialEq, Clone, Copy, Eq, Debug)]
enum ContextMenuAction {
    Kill,
    Term,
}

impl ContextMenuAction {
    fn menu<'a>() -> Option<Vec<widget::menu::Tree<'a, Message>>> {
        Some(widget::menu::items(
            &std::collections::HashMap::new(),
            vec![
                widget::menu::Item::Button("Terminate", ContextMenuAction::Term),
                widget::menu::Item::Divider,
                widget::menu::Item::Button("Kill", ContextMenuAction::Kill),
            ],
        ))
    }
}

impl widget::menu::Action for ContextMenuAction {
    type Message = Message;
    fn message(&self) -> Self::Message {
        match self {
            ContextMenuAction::Kill => Message::ProcessKillActive,
            ContextMenuAction::Term => Message::ProcessTermActive,
        }
    }
}
