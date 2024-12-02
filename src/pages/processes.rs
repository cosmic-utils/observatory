mod category;
mod process;

use crate::app::message::AppMessage;
use category::{Category, CategoryList, Sort};
use cosmic::iced_widget;
use cosmic::{iced, Task};
use cosmic::{
    iced::{alignment::Vertical, Length},
    widget, Element,
};

pub use super::Page;

pub struct ProcessPage {
    sort_data: (Category, Sort),
    users: sysinfo::Users,
    categories: CategoryList,
    processes: process::ProcessList,
    selected_process: Option<sysinfo::Pid>,
    apps: Vec<cosmic::desktop::DesktopEntryData>,
}

impl Page for ProcessPage {
    fn update(
        &mut self,
        sys: &sysinfo::System,
        message: crate::app::message::AppMessage,
    ) -> cosmic::Task<cosmic::app::message::Message<crate::app::message::AppMessage>> {
        let mut tasks = vec![];
        match message {
            AppMessage::ProcessTermActive => {
                sys.process(self.selected_process.unwrap())
                    .unwrap()
                    .kill_with(sysinfo::Signal::Term)
                    .unwrap();
                self.selected_process = None;
            }
            AppMessage::ProcessKillActive => {
                sys.process(self.selected_process.unwrap()).unwrap().kill();
                self.selected_process = None;
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
            }
            AppMessage::Refresh => {
                self.processes
                    .update(&self.categories, sys, &self.apps, &self.users);
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

        widget::container(
            col.push(process_group_scroll)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .into()
    }

    fn footer(&self) -> Option<Element<'_, AppMessage>> {
        let theme = cosmic::theme::active();
        let cosmic = theme.cosmic();

        let mut row = widget::row::with_capacity(4)
            .align_y(Vertical::Center)
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
    pub fn new() -> Self {
        let users = sysinfo::Users::new_with_refreshed_list();
        let categories = CategoryList::new();
        let processes = process::ProcessList::new();
        let apps = cosmic::desktop::load_applications(None, true);
        Self {
            sort_data: (Category::Name, Sort::Descending),
            users,
            categories,
            processes,
            selected_process: None,
            apps,
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
            &std::collections::HashMap::new(),
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
