mod category;
mod process;

use cosmic::iced;
use cosmic::iced_widget;
use cosmic::{
    iced::{alignment::Vertical, Length},
    widget, Element,
};

use crate::app::message::Message;
use category::{Category, CategoryList, Sort};

pub struct ProcessPage {
    sort_data: (Category, Sort),
    users: sysinfo::Users,
    categories: CategoryList,
    processes: process::ProcessList,
    selected_process: Option<sysinfo::Pid>,
    apps: Vec<cosmic::desktop::DesktopEntryData>,
}

impl ProcessPage {
    pub fn new(sys: &sysinfo::System) -> Self {
        let users = sysinfo::Users::new_with_refreshed_list();
        let categories = CategoryList::new();
        let apps = cosmic::desktop::load_applications(None, true);
        let processes = process::ProcessList::new(&categories, sys, &apps, &users);
        Self {
            sort_data: (Category::Name, Sort::Descending),
            users,
            categories,
            processes,
            selected_process: None,
            apps,
        }
    }

    pub fn proc_info(&self, sys: &sysinfo::System) -> Element<Message> {
        widget::column::with_children(vec![
            widget::text::heading(format!(
                "PID: {}",
                sys.process(self.selected_process.unwrap()).unwrap().pid()
            ))
            .into(),
            widget::text::heading(format!(
                "Parent PID: {}",
                sys.process(self.selected_process.unwrap())
                    .unwrap()
                    .parent()
                    .unwrap()
            ))
            .into(),
        ])
        .into()
    }

    pub fn update(
        &mut self,
        sys: &sysinfo::System,
        message: Message,
    ) -> cosmic::app::Task<Message> {
        let mut tasks = vec![];
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
                if self.selected_process == pid {
                    tasks.push(cosmic::app::command::message(cosmic::app::Message::App(
                        Message::ToggleContextPage(crate::app::context::ContextPage::ProcInfo),
                    )));
                } else {
                    self.selected_process = pid;
                }
            }
            Message::ProcessCategoryClick(index) => {
                let cat = Category::from_index(index);
                if cat == self.sort_data.0 {
                    self.sort_data.1.opposite();
                } else {
                    self.sort_data = (cat, Sort::Descending);
                }
            }
            Message::Refresh => {
                self.update_processes(sys);
            }
            Message::KeyPressed(key) => {
                if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                    self.selected_process = None;
                }
            }
            _ => {}
        };

        cosmic::Task::batch(tasks)
    }

    pub fn view(&self) -> Element<Message> {
        let theme = cosmic::theme::active();

        // The vertical column of process elements
        let col = widget::column::with_children(vec![
            self.categories.element(&theme, &self.sort_data),
            iced_widget::horizontal_rule(1)
                .width(Length::Fixed(800.))
                .into(),
        ]);

        let process_column =
            self.processes
                .element(&theme, &self.selected_process, &self.sort_data);
        // Push process rows into scrollable widget
        let process_group_scroll = widget::context_menu(
            iced_widget::Scrollable::with_direction(
                process_column,
                iced_widget::scrollable::Direction::Vertical(
                    iced_widget::scrollable::Scrollbar::default(),
                ),
            )
            .width(Length::Fill),
            ContextMenuAction::menu(),
        );

        col.push(process_group_scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn footer(&self) -> Option<Element<Message>> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();

        let mut row = widget::row::with_capacity(4)
            .align_y(Vertical::Center)
            .spacing(cosmic.space_xs());
        row = row.push(widget::horizontal_space());
        if self.selected_process.is_some() {
            row =
                row.push(widget::button::destructive("Kill").on_press(Message::ProcessKillActive));
            row = row
                .push(widget::button::suggested("Terminate").on_press(Message::ProcessTermActive));
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

    pub fn update_processes(&mut self, sys: &sysinfo::System) {
        self.processes
            .update(&self.categories, sys, &self.apps, &self.users);
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
