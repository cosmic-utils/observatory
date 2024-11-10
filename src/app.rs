pub mod message;
mod overview;
mod processes;
mod resources;
pub mod cosmic_theming;

pub use cosmic::app::{Core, Settings, Task};
pub use cosmic::iced_core::Size;
use cosmic::widget::container;
pub use cosmic::widget::nav_bar;
use cosmic::widget::text;
pub use cosmic::{executor, iced, ApplicationExt, Element};

use message::Message;

#[derive(Clone, Copy)]
pub enum Page {
    Overview,
    Resources,
    Processes,
}

impl Page {
    const fn as_str(self) -> &'static str {
        match self {
            Page::Overview => "Overview",
            Page::Resources => "Resources",
            Page::Processes => "Processes",
        }
    }
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    nav_model: nav_bar::Model,

    sys: sysinfo::System,
    process_page: processes::ProcessPage,
}

/// Implement [`cosmic::Application`] to integrate with COSMIC.
impl cosmic::Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received [`cosmic::Application::new`].
    type Flags = Vec<Page>;

    /// Message type specific to our [`App`].
    type Message = Message;

    /// The unique application ID to supply to the window manager.
    const APP_ID: &'static str = "org.cosmic.SystemMonitor";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        cosmic::iced::time::every(cosmic::iced::time::Duration::from_secs(1))
            .map(|_| Message::Refresh)
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, input: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav_model = nav_bar::Model::default();

        for title in input {
            nav_model.insert().text(title.as_str()).data(title);
        }

        nav_model.activate_position(0);

        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_all();

        let mut process_page = processes::ProcessPage::new(&sys);
        process_page.update_processes(&sys);

        let mut app = App {
            core,
            nav_model,
            sys,
            process_page,
        };

        let command = app.update_title();
        (app, command)
    }

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav_model.activate(id);
        self.update_title()
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        if message == Message::Refresh {
            self.sys.refresh_all();
        }

        if let Some(&page) = self.nav_model.active_data::<Page>() {
            match page {
                Page::Processes => {
                    self.process_page.update(&self.sys, message);
                }
                _ => {}
            }
        };

        Task::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        match self.nav_model.active_data::<Page>() {
            Some(&page) => match page {
                Page::Overview => container(overview::overview(&self.sys)).into(),
                Page::Resources => container(resources::resources(&self.sys)).into(),
                Page::Processes => container(self.process_page.view()).into(),
            },
            _ => text::body("N/A").into(),
        }
    }

    fn footer(&self) -> Option<Element<Self::Message>> {
        match self.nav_model.active_data::<Page>() {
            Some(&page) => match page {
                Page::Processes => self.process_page.footer(),
                _ => None,
            },
            _ => None,
        }
    }
}

impl App
where
    Self: cosmic::Application,
{
    fn active_page_title(&mut self) -> &str {
        self.nav_model
            .text(self.nav_model.active())
            .unwrap_or("Unknown Page")
    }

    fn update_title(&mut self) -> Task<Message> {
        let header_title = self.active_page_title().to_owned();
        let window_title = format!("{header_title} — COSMIC AppDemo");
        self.set_header_title(header_title);
        self.set_window_title(window_title, self.core.main_window_id().unwrap())
    }
}
