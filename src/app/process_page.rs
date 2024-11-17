mod category;
mod process;

use cosmic::iced;
use cosmic::iced_widget;
use cosmic::{
    iced::{alignment::Vertical, Length},
    widget, Element,
};

use crate::app::applications::Application;
use crate::app::message::Message;
use category::{Category, CategoryList, Sort};
use process::Process;

pub struct ProcessPage {
    sort_data: (Category, Sort),
    users: sysinfo::Users,
    active_uid: sysinfo::Uid,
    categories: CategoryList,
    processes: Vec<Process>,
    selected_process: Option<sysinfo::Pid>,
}

impl ProcessPage {
    pub fn new(sys: &sysinfo::System) -> Self {
        Self {
            sort_data: (Category::Name, Sort::Descending),
            users: sysinfo::Users::new_with_refreshed_list(),
            active_uid: sys
                .process(sysinfo::get_current_pid().unwrap())
                .unwrap()
                .user_id()
                .unwrap()
                .clone(),
            categories: category::CategoryList::new(),
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
            Message::ProcessClick(pid) => self.selected_process = pid,
            Message::ProcessCategoryClick(index) => {
                let cat = Category::from_index(index);
                if cat == self.sort_data.0 {
                    self.sort_data.1.opposite();
                } else {
                    self.sort_data = (cat, Sort::Descending);
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
        let col = widget::column::with_children(vec![
            self.categories.element(&theme, &self.sort_data),
            iced_widget::horizontal_rule(1).into(),
        ])
        .width(Length::Fixed(700.));

        let mut process_column =
            widget::column()
                .spacing(cosmic.space_xxxs())
                .padding([0, cosmic.space_xxs(), 0, 0]);
        for process in &self.processes {
            process_column = process_column
                .push(process.element(&theme, self.selected_process == Some(process.pid)));
        }
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

        iced_widget::Scrollable::with_direction(
            col.push(process_group_scroll),
            iced_widget::scrollable::Direction::Horizontal(
                iced_widget::scrollable::Scrollbar::default(),
            ),
        )
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

    pub fn update_processes(&mut self, sys: &sysinfo::System, apps: &Vec<Application>) {
        self.processes = sys
            .processes()
            .values()
            .filter(|process| {
                process.thread_kind().is_none() && process.user_id() == Some(&self.active_uid)
            })
            .map(|process| Process::from_process(&self.categories, process, apps, sys, &self.users))
            .collect();

        self.sort_processes();
    }

    fn sort_processes(&mut self) {
        self.processes.sort_by(|a, b| match self.sort_data.1 {
            Sort::Ascending => Process::compare(a, b, &self.sort_data.0),
            Sort::Descending => Process::compare(b, a, &self.sort_data.0),
        })
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
