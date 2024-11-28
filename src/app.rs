mod action;
pub mod bindings;
pub mod context;
pub mod flags;
mod menu;
pub mod message;

use crate::core::config::ObservatoryConfig;
use crate::core::icons;
use crate::fl;
use crate::pages::{self, overview, processes, resources};
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
use std::any::TypeId;
use std::collections::HashMap;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};

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
    const APP_ID: &'static str = "io.github.cosmic_utils.observatory";

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
            .data(Box::new(overview::OverviewPage::new()) as Box<dyn pages::Page>);
        nav_model
            .insert()
            .text("Resources")
            .icon(icons::get_icon("speedometer-symbolic", 18))
            .data(Box::new(resources::ResourcePage::new()) as Box<dyn pages::Page>);
        nav_model
            .insert()
            .text("Processes")
            .icon(icons::get_icon("view-list-symbolic", 18))
            .data(Box::new(processes::ProcessPage::new()) as Box<dyn pages::Page>);
        nav_model.activate_position(0);

        let sys = sysinfo::System::new_all();

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
        };

        let command = Task::batch([
            app.update_title(),
            cosmic::app::command::message::app(Message::Refresh),
        ]);
        (app, command)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        match self.context_page {
            ContextPage::About => Some(
                context_drawer::about(&self.about, Message::Open, Message::ContextClose)
                    .title(self.context_page.title()),
            ),
            ContextPage::Settings => Some(
                context_drawer::context_drawer(self.settings(), Message::ContextClose)
                    .title(self.context_page.title()),
            ),
            ContextPage::PageInfo => self
                .nav_model
                .active_data::<Box<dyn pages::Page>>()?
                .context_menu(),
        }
    }

    fn footer(&self) -> Option<Element<Self::Message>> {
        match self.nav_model.active_data::<Box<dyn pages::Page>>() {
            Some(page) => page.footer(),
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
                let sys = &mut self.sys;
                sys.refresh_cpu_all();
                sys.refresh_memory();
                sys.refresh_processes_specifics(
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
        // Get the entity ids
        let entities = self
            .nav_model
            .iter()
            .collect::<Vec<widget::segmented_button::Entity>>();

        for entity in entities {
            let page = self.nav_model.data_mut::<Box<dyn pages::Page>>(entity);
            if let Some(page) = page {
                tasks.push(page.update(&self.sys, message.clone()));
            }
        }

        Task::batch(tasks)
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        if let Some(page) = self.nav_model.active_data::<Box<dyn pages::Page>>() {
            widget::container(page.view())
                .height(cosmic::iced::Length::Fill)
                .into()
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
        self.set_window_title(window_title)
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
