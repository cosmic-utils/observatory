mod action;
pub mod bindings;
pub mod context;
pub mod flags;
mod menu;
pub mod message;

use std::any::TypeId;
use std::collections::HashMap;

use crate::core::config::ObservatoryConfig;
use crate::core::icons;
use crate::fl;
use crate::pages::{overview, processes, resources};
use action::Action;
use bindings::key_binds;
use context::ContextPage;
use cosmic::app::context_drawer;
pub use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{CosmicConfigEntry, Update};
use cosmic::cosmic_theme::ThemeMode;
use cosmic::iced::keyboard::{Key, Modifiers};
use cosmic::iced::{event, keyboard::Event as KeyEvent, Event};
use cosmic::widget;
use cosmic::widget::about::About;
use cosmic::widget::menu::{Action as _, KeyBind};
pub use cosmic::{executor, ApplicationExt, Element};
use message::Message;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};

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
    about: About,
    handler: Option<cosmic::cosmic_config::Config>,
    config: ObservatoryConfig,
    app_themes: Vec<String>,
    modifiers: Modifiers,
    key_binds: HashMap<KeyBind, Action>,
    context_page: ContextPage,
    sys: sysinfo::System,
    overview_page: overview::OverviewPage,
    process_page: processes::ProcessPage,
    resource_page: resources::ResourcePage,
}

/// Implement [`cosmic::Application`] to integrate with COSMIC.
impl cosmic::Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received [`cosmic::Application::new`].
    type Flags = flags::Flags;

    /// Message types specific to our [`App`].
    type Message = Message;

    /// The unique application ID to supply to the window manager.
    const APP_ID: &'static str = "org.cosmic-utils.Observatory";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, _input: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav_model = widget::nav_bar::Model::default();
        nav_model
            .insert()
            .text("Overview")
            .icon(icons::get_icon("user-home-symbolic", 18))
            .data(Page::Overview);
        nav_model
            .insert()
            .text("Resources")
            .icon(icons::get_icon("speedometer-symbolic", 18))
            .data(Page::Resources);
        nav_model
            .insert()
            .text("Processes")
            .icon(icons::get_icon("view-list-symbolic", 18))
            .data(Page::Processes);
        nav_model.activate_position(0);

        let mut sys = sysinfo::System::new_all();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );

        let mut process_page = processes::ProcessPage::new(&sys);
        process_page.update_processes(&sys);

        let resource_page = resources::ResourcePage::new();

        let overview_page = overview::OverviewPage::new();

        let (config, handler) = (
            ObservatoryConfig::config(),
            ObservatoryConfig::config_handler(),
        );

        let app_themes = vec![fl!("match-desktop"), fl!("dark"), fl!("light")];

        let about = About::default()
            .name(fl!("app-title"))
            .icon(Self::APP_ID)
            .version("0.1.0")
            .license("GPL-3.0")
            .author("Adam Cosner")
            .links([
                ("Repository", "https://github.com/cosmic-utils/observatory"),
                (
                    "Support",
                    "https://github.com/cosmic-utils/observatory/issues",
                ),
            ])
            .developers([
                ("Adam Cosner", ""),
                ("Eduardo Flores", "edfloreshz@proton.me"),
            ]);

        let mut app = App {
            core,
            nav_model,
            about,
            handler,
            config,
            app_themes,
            modifiers: Modifiers::empty(),
            key_binds: key_binds(),
            context_page: ContextPage::Settings,
            sys,
            overview_page,
            process_page,
            resource_page,
        };

        let command = app.update_title();
        (app, command)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => {
                context_drawer::about(&self.about, Message::Open, Message::ContextClose)
                    .title(self.context_page.title())
            }
            ContextPage::Settings => {
                context_drawer::context_drawer(self.settings(), Message::ContextClose)
                    .title(self.context_page.title())
            }
            ContextPage::ProcInfo => context_drawer::context_drawer(
                self.process_page.proc_info(&self.sys),
                Message::ContextClose,
            )
            .title(self.context_page.title()),
        })
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

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(&self.key_binds)]
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
        let key_press =
            cosmic::iced_winit::graphics::futures::keyboard::on_key_press(key_to_message);

        struct ConfigSubscription;
        struct ThemeSubscription;

        let keybinds = event::listen_with(|event, _status, _window_id| match event {
            Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => {
                Some(Message::Key(modifiers, key))
            }
            Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                Some(Message::Modifiers(modifiers))
            }
            _ => None,
        });

        let config = cosmic::cosmic_config::config_subscription(
            TypeId::of::<ConfigSubscription>(),
            Self::APP_ID.into(),
            ObservatoryConfig::VERSION,
        )
        .map(|update: Update<ThemeMode>| {
            if !update.errors.is_empty() {
                log::info!(
                    "errors loading config {:?}: {:?}",
                    update.keys,
                    update.errors
                );
            }
            Message::SystemThemeChanged
        });
        let theme = cosmic::cosmic_config::config_subscription::<_, ThemeMode>(
            TypeId::of::<ThemeSubscription>(),
            cosmic::cosmic_theme::THEME_MODE_ID.into(),
            ThemeMode::version(),
        )
        .map(|update: Update<ThemeMode>| {
            if !update.errors.is_empty() {
                log::info!(
                    "errors loading theme mode {:?}: {:?}",
                    update.keys,
                    update.errors
                );
            }
            Message::SystemThemeChanged
        });

        cosmic::iced::Subscription::batch([update_clock, key_press, keybinds, config, theme])
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let mut tasks = vec![];
        match message {
            Message::Refresh => {
                self.sys.refresh_cpu_all();
                self.sys.refresh_memory();
                self.sys.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    true,
                    ProcessRefreshKind::everything(),
                );
            }
            Message::SystemThemeChanged => tasks.push(self.update_theme()),
            Message::Open(ref url) => {
                if let Err(err) = open::that_detached(url) {
                    log::error!("Failed to open URL: {}", err);
                }
            }
            Message::ToggleContextPage(ref context_page) => {
                if &self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page.clone();
                    self.core.window.show_context = true;
                }
            }
            Message::ContextClose => self.core.window.show_context = false,
            Message::Key(modifiers, ref key) => {
                for (key_bind, action) in &self.key_binds {
                    if key_bind.matches(modifiers, key) {
                        return self.update(action.message());
                    }
                }
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::AppTheme(theme) => {
                if let Some(ref handler) = self.handler {
                    if let Err(err) = self.config.set_app_theme(handler, theme.into()) {
                        log::error!("Failed to set app theme: {}", err);
                    }
                }
            }
            _ => (),
        }
        tasks.push(self.process_page.update(&self.sys, message.clone()));
        self.resource_page.update(&self.sys, message.clone());
        self.overview_page.update(&self.sys, message.clone());

        Task::batch(tasks)
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        if let Some(page) = self.nav_model.active_data::<Page>() {
            match page {
                Page::Overview => widget::container(self.overview_page.view()).into(),
                Page::Resources => widget::container(self.resource_page.view(&self.sys)).into(),
                Page::Processes => widget::container(self.process_page.view()).into(),
            }
        } else {
            widget::text("ERROR, PAGE NOT SET").into()
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
        let window_title = format!("{header_title} — COSMIC Observatory");
        self.set_header_title(header_title);
        self.set_window_title(window_title, self.core.main_window_id().unwrap())
    }

    fn update_theme(&self) -> Task<Message> {
        cosmic::app::command::set_theme(self.config.app_theme.theme())
    }

    fn settings(&self) -> Element<Message> {
        widget::scrollable(widget::settings::section().title(fl!("appearance")).add(
            widget::settings::item::item(
                fl!("theme"),
                widget::dropdown(
                    &self.app_themes,
                    Some(self.config.app_theme.into()),
                    |theme| Message::AppTheme(theme),
                ),
            ),
        ))
        .into()
    }
}

fn key_to_message(key: Key, _: Modifiers) -> Option<Message> {
    Some(Message::KeyPressed(key))
}
