use std::collections::HashMap;
use std::path::Path;

use super::category::{Category, CategoryList, Sort};
use crate::app::message::Message;
use cosmic::iced::{alignment::Vertical, Length};
use cosmic::{theme, widget, Element};
use sysinfo::Users;

#[derive(Debug)]
struct RunningFlatpak {
    pid: sysinfo::Pid,
    application: String,
}

pub struct ProcessList {
    process_map: HashMap<sysinfo::Pid, Process>,
    root_processes: Vec<sysinfo::Pid>,
    running_flatpaks: Vec<RunningFlatpak>,
}

impl ProcessList {
    pub fn new() -> ProcessList {
        let process_list = ProcessList {
            process_map: HashMap::new(),
            root_processes: Vec::new(),
            running_flatpaks: Vec::new(),
        };

        process_list
    }

    pub fn update(
        &mut self,
        categories: &CategoryList,
        sys: &sysinfo::System,
        apps: &Vec<cosmic::desktop::DesktopEntryData>,
        users: &Users,
    ) {
        self.load_flatpaks();
        self.load_processes(categories, sys, apps, users);
    }

    fn load_flatpaks(&mut self) {
        let flatpak_ps_output = std::process::Command::new("flatpak")
            .arg("ps")
            .arg("--ostree-verbose")
            .arg("--columns=pid,application")
            .output();
        if let Ok(output) = flatpak_ps_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let iter = stdout.lines();
            for flatpak in iter {
                let mut iter = flatpak.split_whitespace();
                let pid = if let Some(pid) = iter.next() {
                    sysinfo::Pid::from(pid.parse::<usize>().unwrap())
                } else {
                    log::error!("Could not parse flatpak pid: {}", flatpak);
                    sysinfo::Pid::from(0)
                };
                let application = if let Some(application) = iter.next() {
                    application
                } else {
                    log::error!("Could not parse flatpak application: {}", flatpak);
                    ""
                }
                .to_string();
                self.running_flatpaks
                    .push(RunningFlatpak { pid, application });
            }
        }
    }

    fn load_processes(
        &mut self,
        categories: &CategoryList,
        sys: &sysinfo::System,
        apps: &Vec<cosmic::desktop::DesktopEntryData>,
        users: &Users,
    ) {
        let active_uid = sys
            .process(sysinfo::get_current_pid().unwrap())
            .unwrap()
            .user_id()
            .unwrap();
        self.process_map.clear();
        self.root_processes.clear();

        for process in sys
            .processes()
            .values()
            .filter(|proc| proc.thread_kind().is_none() && proc.user_id().is_some())
        {
            if process.user_id().unwrap() == active_uid {
                // Create the process and insert into the hashmap
                let proc = Process::from_process(
                    categories,
                    process,
                    apps,
                    &self.running_flatpaks,
                    sys,
                    &users,
                );
                self.process_map.insert(process.pid(), proc);
                // If the process has root pid 1, then it's a "root" process and its icon should propagate down
                if let Some(parent) = process.parent() {
                    if parent == sysinfo::Pid::from(1) {
                        self.root_processes.push(process.pid());
                    }
                }
            }
        }
    }

    pub fn element(
        &self,
        theme: &theme::Theme,
        selected_process: &Option<sysinfo::Pid>,
        sort: &(Category, Sort),
    ) -> Element<Message> {
        let cosmic = theme.cosmic();
        let mut process_column =
            widget::column()
                .spacing(cosmic.space_xxxs())
                .padding([0, cosmic.space_xxs(), 0, 0]);
        let mut sorted_processes = self.process_map.values().collect::<Vec<&Process>>();
        sorted_processes.sort_by(|a, b| match sort.1 {
            Sort::Ascending => Process::compare(&a, &b, &sort.0),
            Sort::Descending => Process::compare(&b, &a, &sort.0),
        });
        for process in sorted_processes {
            let element = process.element(&theme, selected_process == &Some(process.pid));
            process_column = process_column.push(element);
        }

        process_column.into()
    }
}

#[derive(Clone)]
struct DataPoint {
    icon: Option<cosmic::desktop::IconSource>,
    content: String,
    category: Category,
}

#[derive(Clone)]
pub struct Process {
    icon: cosmic::desktop::IconSource,
    data_points: Vec<DataPoint>,
    cpu_percent: f32,
    mem_bytes: u64,
    disk_bytes: u64,
    pub pid: sysinfo::Pid,
}

// impl Default for Process {
//     fn default() -> Self {
//         Self {
//             icon: cosmic::desktop::IconSource::default(),
//             data_points: Vec::new(),
//             cpu_percent: 0.,
//             mem_bytes: 0,
//             disk_bytes: 0,
//             pid: sysinfo::Pid::from(0),
//             children: Vec::new(),
//         }
//     }
// }

impl Process {
    fn from_process(
        categories: &CategoryList,
        process: &sysinfo::Process,
        apps: &Vec<cosmic::desktop::DesktopEntryData>,
        flatpaks: &Vec<RunningFlatpak>,
        sys: &sysinfo::System,
        users: &Users,
    ) -> Self {
        let mut res = Self {
            icon: Process::get_icon(process, sys, apps, flatpaks),
            data_points: Vec::new(),
            cpu_percent: process.cpu_usage() / sys.cpus().len() as f32,
            mem_bytes: process.memory(),
            disk_bytes: process.disk_usage().read_bytes + process.disk_usage().written_bytes,
            pid: process.pid(),
        };
        for category in categories.0.iter() {
            res.data_points
                .push(DataPoint::new(category.clone(), &res, process, sys, users));
        }

        res
    }

    fn compare(a: &Process, b: &Process, cat: &Category) -> std::cmp::Ordering {
        match cat {
            Category::Name => {
                let mut ord = b.data_points[Category::Name.index() as usize]
                    .content
                    .to_ascii_lowercase()
                    .cmp(
                        &a.data_points[Category::Name.index() as usize]
                            .content
                            .to_ascii_lowercase(),
                    );
                if ord == std::cmp::Ordering::Equal {
                    ord = a.pid.cmp(&b.pid);
                }
                ord
            }
            Category::Pid => a.pid.cmp(&b.pid),
            Category::User => {
                let mut ord = a.data_points[Category::User.index() as usize]
                    .content
                    .cmp(&b.data_points[Category::User.index() as usize].content);
                if ord == std::cmp::Ordering::Equal {
                    ord = Self::compare(a, b, &Category::Name);
                }
                ord
            }
            Category::Cpu => a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap(),
            Category::Memory => a.mem_bytes.partial_cmp(&b.mem_bytes).unwrap(),
            Category::Disk => {
                let mut ord = a.disk_bytes.partial_cmp(&b.disk_bytes).unwrap();
                if ord == std::cmp::Ordering::Equal {
                    ord = Self::compare(a, b, &Category::Name);
                }
                ord
            }
        }
    }

    fn get_name(process: &sysinfo::Process) -> String {
        // Check if the cmd file name starts with process.data[Category::Name.index() as usize].data()
        let name = process.name().to_str().unwrap();
        let cmd = process
            .exe()
            .unwrap_or(Path::new("/"))
            .file_name()
            .unwrap_or(std::ffi::OsStr::new(process.name()));

        let file_name = Path::new(cmd).file_name();
        if let Some(file_name) = file_name {
            let file_name = file_name.to_str().unwrap().split(' ').nth(0).unwrap();
            // Now that we've established the cmd, let's check that name starts with it!
            return file_name.to_owned();
        }
        name.into()
    }

    fn get_icon(
        process: &sysinfo::Process,
        sys: &sysinfo::System,
        apps: &Vec<cosmic::desktop::DesktopEntryData>,
        flatpaks: &Vec<RunningFlatpak>,
    ) -> cosmic::desktop::IconSource {
        let cmd = process.cmd();

        // Flatpak handling
        for flatpak in flatpaks {
            if flatpak.pid == process.pid() {
                if let Some(app) = apps.iter().find(|app| app.id == flatpak.application) {
                    return app.icon.clone();
                }
            }
        }

        for app in apps {
            let exec = app.exec.clone().unwrap_or_default();
            let mut exec = shlex::Shlex::new(exec.as_ref());

            let executable = match exec.next() {
                Some(executable) if !executable.contains('=') => executable,
                _ => "NoExec".into(),
            };

            let no_cmd = "NoCmd".into();
            let cmd_start = cmd.iter().nth(0).unwrap_or(&no_cmd).to_string_lossy();
            if cmd_start == executable {
                return app.icon.clone();
            }
        }

        // If it reaches this point, it didn't find an icon, so check its parent
        if let Some(parent_pid) = process.parent() {
            if parent_pid != sysinfo::Pid::from(1) {
                let parent = sys.process(parent_pid).unwrap();
                return Self::get_icon(parent, sys, apps, flatpaks);
            }
        }
        cosmic::desktop::IconSource::default()
    }

    fn element(&self, theme: &cosmic::Theme, is_selected: bool) -> Element<'_, Message> {
        let data = &self.data_points;
        let mut row = widget::row::with_capacity::<Message>(6);
        for dp in data {
            row = row.push(dp.element(theme));
        }

        // Create the button widget
        widget::button::custom(widget::container(row).width(Length::Shrink))
            .class(if is_selected {
                theme::Button::Suggested
            } else {
                theme::Button::HeaderBar
            })
            .padding([0, 0])
            .on_press(Message::ProcessClick(Some(self.pid)))
            .into()
    }
}

impl DataPoint {
    fn new(
        category: Category,
        process: &Process,
        sys_process: &sysinfo::Process,
        sys: &sysinfo::System,
        users: &Users,
    ) -> Self {
        match category {
            Category::Name => {
                let icon = Some(process.icon.clone());
                let content = Process::get_name(sys_process);
                DataPoint {
                    icon,
                    content,
                    category,
                }
            }
            Category::Pid => {
                let content = sys_process.pid().to_string();
                DataPoint {
                    icon: None,
                    content,
                    category,
                }
            }
            Category::User => {
                let content = users
                    .get_user_by_id(sys_process.user_id().unwrap())
                    .unwrap()
                    .name()
                    .into();
                DataPoint {
                    icon: None,
                    content,
                    category,
                }
            }
            Category::Cpu => {
                let content = format!("{:.1}%", sys_process.cpu_usage() / sys.cpus().len() as f32);
                DataPoint {
                    icon: None,
                    content,
                    category,
                }
            }
            Category::Memory => {
                let content = {
                    let bytes = sys_process.memory();
                    if bytes < 1024u64.pow(1) {
                        format!("{} B", bytes)
                    } else if bytes < 1024u64.pow(2) {
                        format!("{:.2} KiB", bytes as f64 / 1024f64.powf(1.))
                    } else if bytes < 1024u64.pow(3) {
                        format!("{:.2} MiB", bytes as f64 / 1024f64.powf(2.))
                    } else {
                        format!("{:.2} GiB", bytes as f64 / 1024f64.powf(3.))
                    }
                };
                DataPoint {
                    icon: None,
                    content,
                    category,
                }
            }
            Category::Disk => {
                let content = {
                    let bytes = sys_process.disk_usage().read_bytes
                        + sys_process.disk_usage().written_bytes;
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
                };
                DataPoint {
                    icon: None,
                    content,
                    category,
                }
            }
        }
    }

    fn element(&self, theme: &theme::Theme) -> Element<'_, Message> {
        let cosmic = theme.cosmic();

        let icon: Option<cosmic::desktop::IconSource> = self.icon.clone();
        let content: String = self.content.clone();
        let width = self.category.width();
        let align = self.category.alignment();

        let data_row = widget::row::with_children(match icon {
            Some(icon) => vec![
                icon.as_cosmic_icon().size(24).into(),
                widget::text::body(content).into(),
            ],
            None => vec![widget::text::body(content).into()],
        })
        .spacing(cosmic.space_xxs())
        .align_y(Vertical::Center);

        widget::container(data_row)
            .padding([cosmic.space_xxs(), cosmic.space_xs()])
            .width(width)
            .align_x(align)
            .into()
    }
}
