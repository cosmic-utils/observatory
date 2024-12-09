use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, Config, CosmicConfigEntry},
    Application,
};
use serde::{Deserialize, Serialize};

use crate::app::App;

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct ObservatoryConfig {
    pub app_theme: AppTheme,
}

impl ObservatoryConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(App::APP_ID, Self::VERSION).ok()
    }

    pub fn config() -> ObservatoryConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                ObservatoryConfig::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    log::error!("errors loading config: {:?}", errs);
                    config
                })
            }
            None => ObservatoryConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    #[default]
    System,
    Dark,
    Light,
}

impl From<usize> for AppTheme {
    fn from(value: usize) -> Self {
        match value {
            1 => AppTheme::Dark,
            2 => AppTheme::Light,
            _ => AppTheme::System,
        }
    }
}

impl From<AppTheme> for usize {
    fn from(value: AppTheme) -> Self {
        match value {
            AppTheme::System => 0,
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
        }
    }
}

impl AppTheme {
    pub fn theme(&self) -> cosmic::theme::Theme {
        match self {
            Self::Dark => {
                let mut t = cosmic::theme::system_dark();
                t.theme_type.prefer_dark(Some(true));
                t
            }
            Self::Light => {
                let mut t = cosmic::theme::system_light();
                t.theme_type.prefer_dark(Some(false));
                t
            }
            Self::System => cosmic::theme::system_preference(),
        }
    }
}
