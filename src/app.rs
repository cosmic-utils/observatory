pub mod message;
mod overview;
mod processes;
mod resources;

pub use cosmic::app::{Command, Core, Settings};
use cosmic::iced::time::every;
pub use cosmic::iced_core::Size;
pub use cosmic::widget::nav_bar;
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
    fn init(core: Core, input: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut nav_model = nav_bar::Model::default();

        for title in input {
            nav_model.insert().text(title.as_str()).data(title);
        }

        nav_model.activate_position(0);

        let sys = sysinfo::System::new_all();
        let process_page = processes::ProcessPage::new();

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
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<Self::Message> {
        self.nav_model.activate(id);
        self.update_title()
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        if message == Message::Refresh {
            self.sys.refresh_all();
        }

        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        match self.nav_model.active_data::<Page>() {
            Some(&page) => match page {
                Page::Overview => {
                    Element::from(cosmic::widget::container(overview::overview(&self.sys)))
                }
                Page::Resources => {
                    Element::from(cosmic::widget::container(resources::resources(&self.sys)))
                }
                Page::Processes => Element::from(cosmic::widget::container(
                    self.process_page.processes(&self.sys),
                )),
            },
            _ => Element::from(cosmic::widget::text("N/A")),
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

    fn update_title(&mut self) -> Command<Message> {
        let header_title = self.active_page_title().to_owned();
        let window_title = format!("{header_title} — COSMIC AppDemo");
        self.set_header_title(header_title);
        self.set_window_title(window_title)
    }
}
