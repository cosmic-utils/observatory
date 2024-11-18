use crate::app::message::Message;
use crate::pages::processes::category::{Category, CategoryList};
use cosmic::iced::{alignment::Vertical, Length};
use cosmic::{theme, widget, Element};

struct DataPoint {
    icon: Option<cosmic::desktop::IconSource>,
    content: String,
    category: Category,
}

pub struct Process {
    data_points: Vec<DataPoint>,
    cpu_percent: f32,
    mem_bytes: u64,
    disk_bytes: u64,
    pub pid: sysinfo::Pid,
}

impl Process {
    pub fn from_process(
        categories: &CategoryList,
        process: &sysinfo::Process,
        apps: &Vec<cosmic::desktop::DesktopEntryData>,
        sys: &sysinfo::System,
        users: &sysinfo::Users,
    ) -> Self {
        let mut data: Vec<DataPoint> = Vec::new();

        for category in categories.0.iter() {
            data.push(DataPoint::new(category.clone(), process, apps, sys, users));
        }

        Self {
            data_points: data,
            cpu_percent: process.cpu_usage() / sys.cpus().len() as f32,
            mem_bytes: process.memory(),
            disk_bytes: process.disk_usage().read_bytes + process.disk_usage().written_bytes,
            pid: process.pid(),
        }
    }

    pub fn compare(a: &Process, b: &Process, cat: &Category) -> std::cmp::Ordering {
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

    fn get_icon(
        process: &sysinfo::Process,
        apps: &Vec<cosmic::desktop::DesktopEntryData>,
    ) -> cosmic::desktop::IconSource {
        let cmd = process.cmd();
        let mut icon = cosmic::desktop::IconSource::default();

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
                icon = app.icon.clone();
            }
        }

        icon
    }

    pub fn element(&self, theme: &cosmic::Theme, is_selected: bool) -> Element<Message> {
        let row = widget::row::with_children::<Message>(
            self.data_points
                .iter()
                .map(|dp| dp.element(theme))
                .collect(),
        );

        // Create the button widget
        widget::button::custom(widget::container(row).width(Length::Shrink))
            .class(if is_selected {
                cosmic::theme::Button::Suggested
            } else {
                cosmic::theme::Button::HeaderBar
            })
            .padding([0, 0])
            .on_press(Message::ProcessClick(Some(self.pid)))
            .into()
    }
}

impl DataPoint {
    fn new(
        category: Category,
        process: &sysinfo::Process,
        apps: &Vec<cosmic::desktop::DesktopEntryData>,
        sys: &sysinfo::System,
        users: &sysinfo::Users,
    ) -> Self {
        match category {
            Category::Name => {
                let icon = Some(Process::get_icon(process, apps));
                let content = Process::get_name(process);
                DataPoint {
                    icon,
                    content,
                    category,
                }
            }
            Category::User => {
                let content = users
                    .get_user_by_id(process.user_id().unwrap())
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
                let content = format!("{:.1}%", process.cpu_usage() / sys.cpus().len() as f32);
                DataPoint {
                    icon: None,
                    content,
                    category,
                }
            }
            Category::Memory => {
                let content = {
                    let bytes = process.memory();
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
                };
                DataPoint {
                    icon: None,
                    content,
                    category,
                }
            }
        }
    }

    fn element(&self, theme: &theme::Theme) -> Element<Message> {
        let cosmic = theme.cosmic();

        let data_row = widget::row::with_children(match &self.icon {
            Some(icon) => vec![
                icon.as_cosmic_icon().size(24).into(),
                widget::text::body(&self.content).into(),
            ],
            None => vec![widget::text::body(&self.content).into()],
        })
        .spacing(cosmic.space_xxs())
        .align_y(Vertical::Center);

        widget::container(data_row)
            .padding([cosmic.space_xxs(), cosmic.space_xs()])
            .width(self.category.width())
            .align_x(self.category.alignment())
            .into()
    }
}
