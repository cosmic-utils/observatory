use crate::app::applications::Application;
use crate::app::message::Message;
use crate::app::process_page::category::{Category, CategoryList};
use cosmic::iced::{alignment::Vertical, Length};
use cosmic::{theme, widget, Element};

#[derive(Clone, Debug)]
struct DataPoint {
    icon_name: Option<String>,
    data: String,
    category: Category,
}

#[derive(Clone, Debug)]
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
        apps: &Vec<Application>,
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
                    .data
                    .to_ascii_lowercase()
                    .cmp(
                        &a.data_points[Category::Name.index() as usize]
                            .data
                            .to_ascii_lowercase(),
                    );
                if ord == std::cmp::Ordering::Equal {
                    ord = a.pid.cmp(&b.pid);
                }
                ord
            }
            Category::User => {
                let mut ord = a.data_points[Category::User.index() as usize]
                    .data
                    .cmp(&b.data_points[Category::User.index() as usize].data);
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
        apps: &Vec<Application>,
        sys: &sysinfo::System,
        users: &sysinfo::Users,
    ) -> Self {
        match category {
            Category::Name => {
                let icon_name = Some(
                    match apps.iter().find(|app| {
                        if let Some(cmd) = process.cmd().iter().nth(0) {
                            app.cmd() == cmd
                        } else {
                            false
                        }
                    }) {
                        Some(app) => app.icon().into(),
                        None => "application-default-symbolic".into(),
                    },
                );
                let data = Process::get_name(process);
                DataPoint {
                    icon_name,
                    data,
                    category,
                }
            }
            Category::User => {
                let user = users
                    .get_user_by_id(process.user_id().unwrap())
                    .unwrap()
                    .name()
                    .into();
                DataPoint {
                    icon_name: None,
                    data: user,
                    category,
                }
            }
            Category::Cpu => {
                let data = format!("{:.1}%", process.cpu_usage() / sys.cpus().len() as f32);
                DataPoint {
                    icon_name: None,
                    data,
                    category,
                }
            }
            Category::Memory => {
                let data = {
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
                    icon_name: None,
                    data,
                    category,
                }
            }
            Category::Disk => {
                let data = {
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
                    icon_name: None,
                    data,
                    category,
                }
            }
        }
    }

    fn element(&self, theme: &theme::Theme) -> Element<Message> {
        let cosmic = theme.cosmic();

        let data_row = widget::row::with_children(match &self.icon_name {
            Some(icon_name) => vec![
                widget::icon::from_name(icon_name.as_str()).into(),
                widget::text::body(&self.data).into(),
            ],
            None => vec![widget::text::body(&self.data).into()],
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
