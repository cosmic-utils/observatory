pub mod message;
pub mod cosmic_theming;
pub mod flags;
mod overview;
mod processes;
mod resources;
pub mod applications;

pub use cosmic::app::{Core, Task};
use cosmic::widget;
pub use cosmic::{executor, ApplicationExt, Element};
use cosmic::iced::keyboard::{Key, Modifiers};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};
use message::Message;

#[derive(Clone, Copy)]
pub enum Page {
    Overview,
    Resources,
    Processes,
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    nav_model: widget::nav_bar::Model,

    apps: Vec<applications::Application>,

    sys: sysinfo::System,
    process_page: processes::ProcessPage,
}

/// Implement [`cosmic::Application`] to integrate with COSMIC.
impl cosmic::Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received [`cosmic::Application::new`].
    type Flags = flags::Flags;

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

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, _input: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav_model = widget::nav_bar::Model::default();
        nav_model.insert().text("Overview").data(Page::Overview);
        nav_model.insert().text("Resources").data(Page::Resources);
        nav_model.insert().text("Processes").data(Page::Processes);
        nav_model.activate_position(2);

        let apps = applications::Application::scan_all();

        let mut sys = sysinfo::System::new_all();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::everything());

        let mut process_page = processes::ProcessPage::new(&sys);
        process_page.update_processes(&sys, &apps);

        let mut app = App {
            core,
            nav_model,
            apps,
            sys,
            process_page,
        };

        let command = app.update_title();
        (app, command)
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

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&widget::nav_bar::Model> {
        Some(&self.nav_model)
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(&mut self, id: widget::nav_bar::Id) -> Task<Self::Message> {
        self.nav_model.activate(id);
        self.update_title()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        let update_clock = cosmic::iced::time::every(cosmic::iced::time::Duration::from_secs(1))
            .map(|_| Message::Refresh);
        let key_press = cosmic::iced_winit::graphics::futures::keyboard::on_key_press(key_to_message);

        cosmic::iced::Subscription::batch([update_clock, key_press])
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        if message == Message::Refresh {
            self.sys.refresh_cpu_all();
            self.sys.refresh_memory();
            self.sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::everything());
        }

        if message == Message::KeyPressed(Key::Named(cosmic::iced::keyboard::key::Named::Escape)) {
            self.core.set_show_context(false);
        }

        if let Some(&page) = self.nav_model.active_data::<Page>() {
            match page {
                Page::Processes => {
                    self.process_page.update(&self.sys, message, &self.apps);
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
                Page::Overview => widget::container(overview::overview(&self.sys)).into(),
                Page::Resources => widget::container(resources::resources(&self.sys)).into(),
                Page::Processes => widget::container(self.process_page.view()).into(),
            },
            _ => widget::text::body("").into(),
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

fn key_to_message(key: Key, _: Modifiers) -> Option<Message> {
    Some(Message::KeyPressed(key))
}
