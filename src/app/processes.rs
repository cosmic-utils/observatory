use crate::app::message::Message;

use cosmic::{
    iced_widget::{horizontal_rule, row, scrollable},
    widget::{container, icon, text::heading},
    Element,
};
use sysinfo::Users;

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

pub struct ProcessPage {
    sort_data: (HeaderCategory, SortDirection),
}

impl ProcessPage {
    pub fn new() -> ProcessPage {
        ProcessPage {
            sort_data: (HeaderCategory::Name, SortDirection::Descending),
        }
    }

    pub fn processes(&self, sys: &sysinfo::System) -> Element<Message> {
        let cosmic::cosmic_theme::Spacing {
            space_s,
            space_xxs,
            space_xxxs,
            ..
        } = cosmic::theme::active().cosmic().spacing;

        let users = Users::new_with_refreshed_list();

        // The vertical column of process elements
        let mut main_column = cosmic::widget::column::<Message>();
        // The different widths of elements
        let (name_width, user_width, cpu_width, memory_width, disk_width) =
            (300, 100, 75, 100, 150);

        // Label row
        main_column = main_column
            .push(row![
                // Name label
                container(if self.sort_data.0 == HeaderCategory::Name {
                    if self.sort_data.1 == SortDirection::Descending {
                        row![
                            container(heading("Name")).padding([0, space_s]),
                            container(icon::from_name("pan-down-symbolic"))
                                .padding([0, space_xxxs])
                        ]
                    } else {
                        row![
                            container(heading("Name")).padding([0, space_s]),
                            container(icon::from_name("pan-up-symbolic")).padding([0, space_xxxs])
                        ]
                    }
                } else {
                    row![container(heading("Name")).padding([0, space_s])]
                })
                .width(name_width)
                .padding([space_xxs, 0]),
                // User label
                container(if self.sort_data.0 == HeaderCategory::User {
                    if self.sort_data.1 == SortDirection::Descending {
                        row![
                            container(heading("User")).padding([0, space_s]),
                            container(icon::from_name("pan-down-symbolic"))
                                .padding([0, space_xxxs])
                        ]
                    } else {
                        row![
                            container(heading("User")).padding([0, space_s]),
                            container(icon::from_name("pan-up-symbolic")).padding([0, space_xxxs])
                        ]
                    }
                } else {
                    row![container(heading("User")).padding([0, space_s])]
                })
                .width(user_width)
                .padding([space_xxs, 0]),
                // Cpu label
                container(if self.sort_data.0 == HeaderCategory::Cpu {
                    if self.sort_data.1 == SortDirection::Descending {
                        row![
                            container(heading("CPU")).padding([0, space_s]),
                            container(icon::from_name("pan-down-symbolic"))
                                .padding([0, space_xxxs])
                        ]
                    } else {
                        row![
                            container(heading("CPU")).padding([0, space_s]),
                            container(icon::from_name("pan-up-symbolic")).padding([0, space_xxxs])
                        ]
                    }
                } else {
                    row![container(heading("CPU")).padding([0, space_s])]
                })
                .width(cpu_width)
                .padding([space_xxs, 0]),
                // Memory label
                container(if self.sort_data.0 == HeaderCategory::Memory {
                    if self.sort_data.1 == SortDirection::Descending {
                        row![
                            container(heading("Memory")).padding([0, space_s]),
                            container(icon::from_name("pan-down-symbolic"))
                                .padding([0, space_xxxs])
                        ]
                    } else {
                        row![
                            container(heading("Memory")).padding([0, space_s]),
                            container(icon::from_name("pan-up-symbolic")).padding([0, space_xxxs])
                        ]
                    }
                } else {
                    row![container(heading("Memory")).padding([0, space_s])]
                })
                .width(memory_width)
                .padding([space_xxs, 0]),
                // Disk label
                container(if self.sort_data.0 == HeaderCategory::Disk {
                    if self.sort_data.1 == SortDirection::Descending {
                        row![
                            container(heading("Disk")).padding([0, space_s]),
                            container(icon::from_name("pan-down-symbolic"))
                                .padding([0, space_xxxs])
                        ]
                    } else {
                        row![
                            container(heading("Disk")).padding([0, space_s]),
                            container(icon::from_name("pan-up-symbolic")).padding([0, space_xxxs])
                        ]
                    }
                } else {
                    row![container(heading("Disk")).padding([0, space_s])]
                })
                .width(disk_width)
                .padding([space_xxs, 0])
            ])
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
            process_group = process_group.push(
                container(row![
                    container(cosmic::widget::text(get_process_name(process)))
                        .width(name_width)
                        .padding([space_xxs, 0]),
                    container(cosmic::widget::text(get_process_user(process, &users)))
                        .width(user_width)
                        .padding([space_xxs, 0]),
                    container(cosmic::widget::text(get_process_cpu(
                        process,
                        sys.cpus().len()
                    )))
                    .width(cpu_width)
                    .padding([space_xxs, 0]),
                    container(cosmic::widget::text(get_process_memory(process)))
                        .width(memory_width)
                        .padding([space_xxs, 0]),
                    container(cosmic::widget::text(get_process_disk(process)))
                        .width(disk_width)
                        .padding([space_xxs, 0])
                ])
                .padding([0, space_s]),
            );
        }

        main_column.push(scrollable(process_group)).into()
    }
}
