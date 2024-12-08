mod category;
//mod process;

pub use super::Page;
use crate::app::message::AppMessage;
use crate::core::system_info::{Process, SystemInfo};
use category::{Category, CategoryList, Sort};

use cosmic::{app::Task, cosmic_theme, iced, widget, Element};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct ProcessPage {
    sort_data: (Category, Sort),
    process_map: HashMap<u32, Process>,
    processes: Vec<(Process, cosmic::desktop::IconSource)>,
    selected_process: Option<u32>,

    core_count: f32,
    memory: f32,

    sys: Arc<RwLock<SystemInfo>>,
}

fn get_proc_name(process: &Process) -> &str {
    if !process.exe.as_ref().is_empty() {
        let entry_name = std::path::Path::new(process.exe.as_ref())
            .file_name()
            .map(|name| name.to_str().unwrap_or(process.name.as_ref()))
            .unwrap_or(process.name.as_ref());
        if entry_name.starts_with("wine") {
            if process.cmd.is_empty() {
                process.name.as_ref()
            } else {
                process.cmd[0]
                    .as_ref()
                    .split("\\")
                    .last()
                    .unwrap_or(process.name.as_ref())
                    .split("/")
                    .last()
                    .unwrap_or(process.name.as_ref())
            }
        } else {
            entry_name
        }
    } else {
        process.name.as_ref()
    }
}

fn process_sort(a: &Process, b: &Process, category: &Category) -> std::cmp::Ordering {
    match category {
        Category::Name => {
            let mut ord = get_proc_name(b)
                .to_lowercase()
                .cmp(&get_proc_name(a).to_lowercase());
            if ord == std::cmp::Ordering::Equal {
                ord = process_sort(a, b, &Category::Pid);
            }
            ord
        }
        Category::Pid => a.pid.cmp(&b.pid),
        Category::Cpu => {
            let mut ord = a.usage_stats.cpu_usage.total_cmp(&b.usage_stats.cpu_usage);
            if ord == std::cmp::Ordering::Equal {
                ord = process_sort(a, b, &Category::Name);
            }
            ord
        }
        Category::Gpu => {
            let mut ord = a.usage_stats.gpu_usage.total_cmp(&b.usage_stats.gpu_usage);
            if ord == std::cmp::Ordering::Equal {
                ord = process_sort(a, b, &Category::Name);
            }
            ord
        }
        Category::Memory => {
            let mut ord = a
                .usage_stats
                .memory_usage
                .total_cmp(&b.usage_stats.memory_usage);
            if ord == std::cmp::Ordering::Equal {
                ord = process_sort(a, b, &Category::Name);
            }
            ord
        }
        Category::Disk => {
            let mut ord = a
                .usage_stats
                .disk_usage
                .total_cmp(&b.usage_stats.disk_usage);
            if ord == std::cmp::Ordering::Equal {
                ord = process_sort(a, b, &Category::Name);
            }
            ord
        }
    }
}

impl Page for ProcessPage {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        let mut tasks = vec![];
        match message {
            AppMessage::ProcessTermActive => {
                if let Some(pid) = self.selected_process {
                    if let Ok(sys) = self.sys.write() {
                        sys.terminate_process(pid)
                    }
                    self.selected_process = None;
                }
            }
            AppMessage::ProcessKillActive => {
                if let Some(pid) = self.selected_process {
                    if let Ok(sys) = self.sys.write() {
                        sys.kill_process(pid)
                    }
                    self.selected_process = None;
                }
            }
            AppMessage::ProcessClick(pid) => {
                if self.selected_process == pid {
                    tasks.push(cosmic::task::message(AppMessage::ToggleContextPage(
                        crate::app::context::ContextPage::PageInfo,
                    )));
                } else {
                    self.selected_process = pid;
                }
            }
            AppMessage::ProcessCategoryClick(index) => {
                let cat = Category::from_index(index);
                if cat == self.sort_data.0 {
                    self.sort_data.1.opposite();
                } else {
                    self.sort_data = (cat, Sort::Descending);
                }
                self.processes.sort_by(|a, b| match self.sort_data.1 {
                    Sort::Ascending => process_sort(&a.0, &b.0, &self.sort_data.0),
                    Sort::Descending => process_sort(&b.0, &a.0, &self.sort_data.0),
                })
            }
            AppMessage::SysInfoRefresh => {
                if let Ok(sys) = self.sys.read() {
                    self.core_count = sys.cpu_static_info().logical_cpu_count as f32;
                    self.memory = crate::core::system_info::mem_info::MemInfo::load()
                        .unwrap()
                        .mem_total as f32;

                    self.process_map = sys.processes();
                    self.processes = self
                        .process_map
                        .values()
                        .cloned()
                        .filter(|proc| {
                            let mut current = proc.pid;
                            loop {
                                if let Some(process) = self.process_map.get(&current) {
                                    if let Some(parent) = self.process_map.get(&process.parent) {
                                        if parent.pid == 1 {
                                            return true;
                                        }
                                        current = parent.pid;
                                    } else {
                                        return false;
                                    }
                                } else {
                                    return false;
                                }
                            }
                        })
                        .map(|process| {
                            let apps = sys.apps();
                            let mut icon = cosmic::desktop::IconSource::default();
                            for app in apps.values() {
                                if let Some(app_icon) = app.icon.clone() {
                                    if let Some(_) =
                                        app.pids.iter().find(|pid| **pid == process.pid)
                                    {
                                        icon =
                                            cosmic::desktop::IconSource::Name(app_icon.to_string());
                                    }
                                }
                            }
                            (process, icon)
                        })
                        .collect();

                    self.processes.sort_by(|a, b| match self.sort_data.1 {
                        Sort::Ascending => process_sort(&a.0, &b.0, &self.sort_data.0),
                        Sort::Descending => process_sort(&b.0, &a.0, &self.sort_data.0),
                    });
                }
            }
            AppMessage::KeyPressed(key) => {
                if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                    self.selected_process = None;
                }
            }
            _ => {}
        };

        Task::batch(tasks)
    }

    fn context_menu(&self) -> Option<cosmic::app::context_drawer::ContextDrawer<'_, AppMessage>> {
        Some(
            cosmic::app::context_drawer::context_drawer(
                widget::column::with_children(vec![widget::text::heading(format!(
                    "PID: {}",
                    self.selected_process.unwrap()
                ))
                .into()]),
                AppMessage::ContextClose,
            )
            .title("About Process"),
        )
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();

        let mut header_row = widget::row();
        for category in CategoryList::new().0 {
            if self.sort_data.0 == category {
                header_row = header_row.push(
                    widget::button::custom(
                        widget::row()
                            .spacing(cosmic.space_xxs())
                            .push(widget::text::heading(category.name()))
                            .push(widget::icon::from_name(match self.sort_data.1 {
                                Sort::Ascending => "pan-up-symbolic",
                                Sort::Descending => "pan-down-symbolic",
                            })),
                    )
                    .on_press(AppMessage::ProcessCategoryClick(category.index()))
                    .class(cosmic::style::Button::HeaderBar)
                    .width(category.width()),
                )
            } else {
                header_row = header_row.push(
                    widget::button::custom(widget::text::heading(category.name()))
                        .on_press(AppMessage::ProcessCategoryClick(category.index()))
                        .class(cosmic::style::Button::HeaderBar)
                        .width(category.width()),
                );
            }
        }

        let mut process_list = widget::column()
            .spacing(cosmic.space_xs())
            .width(iced::Length::Fill);
        for process in &self.processes {
            let is_selected = if let Some(selected) = self.selected_process {
                selected == process.0.pid
            } else {
                false
            };
            process_list = process_list.push(
                widget::button::custom(
                    widget::row()
                        .push(
                            widget::container(
                                widget::row()
                                    .spacing(cosmic.space_xxs())
                                    .push(process.1.as_cosmic_icon().size(24))
                                    .push(widget::text::body(
                                        get_proc_name(&process.0).to_string(),
                                    )),
                            )
                            .width(Category::Name.width()),
                        )
                        .push(widget::container(
                            widget::text::body(process.0.pid.to_string())
                                .width(Category::Pid.width()),
                        ))
                        .push(
                            widget::container(widget::text::body(format!(
                                "{}%",
                                (process.0.usage_stats.cpu_usage / self.core_count).round()
                            )))
                            .width(Category::Cpu.width()),
                        )
                        .push(
                            widget::container(widget::text::body(format!(
                                "{}%",
                                (process.0.usage_stats.gpu_usage).round()
                            )))
                            .width(Category::Gpu.width()),
                        )
                        .push(
                            widget::container(widget::text::body(bytes_to_size(
                                process.0.usage_stats.memory_usage as usize,
                            )))
                            .width(Category::Memory.width()),
                        )
                        .push(
                            widget::container(widget::text::body(bytes_to_speed(
                                process.0.usage_stats.disk_usage as usize,
                            )))
                            .width(Category::Disk.width()),
                        ),
                )
                .on_press(AppMessage::ProcessClick(Some(process.0.pid)))
                .class(crate::style::button::ButtonStyle::ListElement(is_selected).into()),
            )
        }

        widget::layer_container(
            widget::column()
                .spacing(cosmic.space_xs())
                .push(header_row)
                .push(iced::widget::horizontal_rule(1))
                .push(widget::scrollable(process_list)),
        )
        .layer(cosmic_theme::Layer::Primary)
        .padding([cosmic.space_s(), cosmic.space_m()])
        .into()
    }

    fn footer(&self) -> Option<Element<'_, AppMessage>> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();

        let mut row = widget::row::with_capacity(4)
            .align_y(iced::Alignment::Center)
            .spacing(cosmic.space_xs());
        row = row.push(widget::horizontal_space());
        if self.selected_process.is_some() {
            row = row
                .push(widget::button::destructive("Kill").on_press(AppMessage::ProcessKillActive));
            row = row.push(
                widget::button::suggested("Terminate").on_press(AppMessage::ProcessTermActive),
            );
        } else {
            row = row.push(widget::button::destructive("Kill"));
            row = row.push(widget::button::suggested("Terminate"));
        }

        Some(
            widget::layer_container(row)
                .layer(cosmic::cosmic_theme::Layer::Primary)
                .padding([cosmic.space_xxs(), cosmic.space_xs()])
                .into(),
        )
    }
}

impl ProcessPage {
    pub fn new(sys: Arc<RwLock<SystemInfo>>) -> Self {
        Self {
            sort_data: (Category::Name, Sort::Descending),
            processes: Vec::new(),
            process_map: HashMap::new(),
            selected_process: None,
            core_count: 0.,
            memory: 0.,
            sys,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Eq, Debug)]
enum ContextMenuAction {
    Kill,
    Term,
}

impl ContextMenuAction {
    fn menu<'a>() -> Option<Vec<widget::menu::Tree<'a, AppMessage>>> {
        Some(widget::menu::items(
            &HashMap::new(),
            vec![
                widget::menu::Item::Button("Terminate", None, ContextMenuAction::Term),
                widget::menu::Item::Divider,
                widget::menu::Item::Button("Kill", None, ContextMenuAction::Kill),
            ],
        ))
    }
}

impl widget::menu::Action for ContextMenuAction {
    type Message = AppMessage;
    fn message(&self) -> Self::Message {
        match self {
            ContextMenuAction::Kill => AppMessage::ProcessKillActive,
            ContextMenuAction::Term => AppMessage::ProcessTermActive,
        }
    }
}

fn bytes_to_size(bytes: usize) -> String {
    if bytes < 1024usize.pow(1) {
        format!("{} B", bytes)
    } else if bytes < 1024usize.pow(2) {
        format!("{:.2} KiB", bytes as f64 / 1024f64.powf(1.))
    } else if bytes < 1024usize.pow(3) {
        format!("{:.2} MiB", bytes as f64 / 1024f64.powf(2.))
    } else {
        format!("{:.2} GiB", bytes as f64 / 1024f64.powf(3.))
    }
}

fn bytes_to_speed(bytes: usize) -> String {
    if bytes < 1024usize.pow(1) {
        format!("{} B/s", bytes)
    } else if bytes < 1024usize.pow(2) {
        format!("{:.2} KiB/s", bytes as f64 / 1024f64.powf(1.))
    } else if bytes < 1024usize.pow(3) {
        format!("{:.2} MiB/s", bytes as f64 / 1024f64.powf(2.))
    } else {
        format!("{:.2} GiB/s", bytes as f64 / 1024f64.powf(3.))
    }
}
