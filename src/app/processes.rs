use crate::app::message::Message;

use cosmic::{
    iced_widget::{horizontal_rule, row, scrollable},
    widget::{container, icon, text, text::heading},
    Element,
};
use sysinfo::Users;

#[derive(PartialEq)]
enum SortDirection {
    Ascending,
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
    fn name(cat: &HeaderCategory) -> &str {
        match cat {
            HeaderCategory::Name => "Name",
            HeaderCategory::User => "User",
            HeaderCategory::Cpu => "CPU",
            HeaderCategory::Memory => "Memory",
            HeaderCategory::Disk => "Disk",
        }
    }

    // (300, 100, 75, 100, 150);
    fn width(cat: &HeaderCategory) -> u16 {
        match cat {
            HeaderCategory::Name => 250,
            HeaderCategory::User => 150,
            HeaderCategory::Cpu => 75,
            HeaderCategory::Memory => 100,
            HeaderCategory::Disk => 150,
        }
    }
}

pub struct ProcessPage {
    sort_data: (HeaderCategory, SortDirection),
}

impl ProcessPage {
    pub fn new() -> ProcessPage {
        ProcessPage {
            sort_data: (HeaderCategory::Name, SortDirection::Descending),
        }
    }

    pub fn processes<'a>(&'a self, sys: &'a sysinfo::System) -> Element<'a, Message> {
        let cosmic::cosmic_theme::Spacing { space_xxs, .. } =
            cosmic::theme::active().cosmic().spacing;

        // The vertical column of process elements
        let mut main_column = cosmic::widget::column::<Message>();
        // The different widths of elements

        // Label row
        main_column = main_column
            .push(container(row![
                create_header_label(&HeaderCategory::Name, &self.sort_data),
                create_header_label(&HeaderCategory::User, &self.sort_data),
                create_header_label(&HeaderCategory::Cpu, &self.sort_data),
                create_header_label(&HeaderCategory::Memory, &self.sort_data),
                create_header_label(&HeaderCategory::Disk, &self.sort_data),
            ]))
            .push(container(horizontal_rule(1)).padding([0, space_xxs]));

        let mut processes = sys
            .processes()
            .values()
            .filter(|process| match process.thread_kind() {
                None => true,
                _ => false,
            })
            .collect::<Vec<&sysinfo::Process>>();

        processes.sort_by(|a, b| match self.sort_data.0 {
            _ => {
                let name_a = get_process_name(a);

                let name_b = get_process_name(b);

                if self.sort_data.1 == SortDirection::Descending {
                    name_a.to_lowercase().cmp(&name_b.to_lowercase())
                } else {
                    name_b.to_lowercase().cmp(&name_a.to_lowercase())
                }
            }
        });

        let mut process_group = cosmic::widget::column::<Message>();
        for process in processes {
            process_group = process_group.push(create_process_row(process, sys));
        }

        main_column.push(scrollable(process_group)).into()
    }
}

fn get_process_name(process: &sysinfo::Process) -> String {
    if let Some(path) = process.exe() {
        path.file_name().unwrap().to_str().unwrap().into()
    } else {
        process.name().to_str().unwrap().into()
    }
}

fn get_process_user(process: &sysinfo::Process, users: &Users) -> String {
    users
        .get_user_by_id(process.user_id().unwrap())
        .unwrap()
        .name()
        .into()
}

fn get_process_cpu(process: &sysinfo::Process, num_cpus: usize) -> String {
    format!("{:.2}%", process.cpu_usage() / num_cpus as f32).into()
}

fn get_process_memory(process: &sysinfo::Process) -> String {
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
}

fn get_process_disk(process: &sysinfo::Process) -> String {
    let bytes = process.disk_usage().read_bytes + process.disk_usage().written_bytes;
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
}

fn create_header_label<'a>(
    category: &'a HeaderCategory,
    sort_data: &'a (HeaderCategory, SortDirection),
) -> Element<'a, Message> {
    let title_text = container(heading(HeaderCategory::name(&category)))
        .padding([0, cosmic::theme::active().cosmic().spacing.space_xs]);
    let arrow: Element<Message> = if category == &sort_data.0 {
        match sort_data {
            (_, SortDirection::Ascending) => container(icon::from_name("pan-up-symbolic"))
                .padding([0, cosmic::theme::active().cosmic().spacing.space_xxxs])
                .into(),
            (_, SortDirection::Descending) => container(icon::from_name("pan-down-symbolic"))
                .padding([0, cosmic::theme::active().cosmic().spacing.space_xxxs])
                .into(),
        }
    } else {
        text("").into()
    };

    let width = HeaderCategory::width(&category);

    container(row![title_text, arrow])
        .width(width)
        .padding([cosmic::theme::active().cosmic().spacing.space_xxs, 0])
        .into()
}

fn create_process_row<'a>(
    process: &'a sysinfo::Process,
    sys: &'a sysinfo::System,
) -> Element<'a, Message> {
    let users = Users::new_with_refreshed_list();
    let padding = [0, cosmic::theme::active().cosmic().spacing.space_xxs];

    let process_name: Element<'a, Message> = container(text(get_process_name(process)))
        .width(HeaderCategory::width(&HeaderCategory::Name))
        .padding(padding)
        .into();
    let process_user: Element<'a, Message> = container(text(get_process_user(process, &users)))
        .width(HeaderCategory::width(&HeaderCategory::User))
        .padding(padding)
        .into();
    let process_cpu: Element<'a, Message> =
        container(text(get_process_cpu(process, sys.cpus().len())))
            .width(HeaderCategory::width(&HeaderCategory::Cpu))
            .padding(padding)
            .into();
    let process_mem: Element<'a, Message> = container(text(get_process_memory(process)))
        .width(HeaderCategory::width(&HeaderCategory::Memory))
        .padding(padding)
        .into();
    let process_disk: Element<'a, Message> = container(text(get_process_disk(process)))
        .width(HeaderCategory::width(&HeaderCategory::Disk))
        .padding(padding)
        .into();

    let process_row = row![
        process_name,
        process_user,
        process_cpu,
        process_mem,
        process_disk
    ];
    container(process_row)
        .padding([cosmic::theme::active().cosmic().spacing.space_xxs, 0])
        .into()
}
