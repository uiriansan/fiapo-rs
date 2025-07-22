use log::{info, warn};
use serde::Deserialize;
use std::fs;
use std::{env, path::Path};
use toml;

#[derive(Deserialize, Debug, Default)]
#[allow(dead_code)]
pub struct FiapoConfig {
    /// Color of the text in the app
    #[serde(default = "text_color")]
    pub text_color: String,
    /// Color of the background of the app
    #[serde(default = "background_color")]
    pub background_color: String,

    /// Reader options
    #[serde(default = "ReaderConfig::new")]
    pub reader: ReaderConfig,
}

impl FiapoConfig {
    pub fn defaults() -> Self {
        Self {
            text_color: text_color(),
            background_color: background_color(),
            reader: ReaderConfig {
                show_bottom_indicator: reader_show_bottom_indicator(),
            },
        }
    }
    /// Parse a .toml file from a given path and mutate the struct
    pub fn parse_config_file(&mut self, config_path: &str) {
        info!("Loading config from {}...", config_path);
        match fs::read_to_string(config_path) {
            Ok(file_contents) => match toml::from_str::<FiapoConfig>(&file_contents) {
                Ok(config) => {
                    *self = config;
                }
                Err(e) => {
                    warn!(
                        "Failed to parse config file in `{}`: {}. Loading defaults...",
                        config_path, e
                    );
                }
            },
            Err(e) => {
                warn!(
                    "Could not read config file in `{}`: {}. Loading defaults...",
                    config_path, e
                );
            }
        }
    }
}

#[derive(Deserialize, Debug, Default)]
#[allow(dead_code)]
pub struct ReaderConfig {
    #[serde(default = "reader_show_bottom_indicator")]
    show_bottom_indicator: bool,
}
impl ReaderConfig {
    pub fn new() -> Self {
        ReaderConfig {
            show_bottom_indicator: reader_show_bottom_indicator(),
        }
    }
}

pub fn resolve_config_path(path: &str) -> Option<String> {
    if let Ok(home) = env::var("HOME") {
        let final_path = path.replace("~", &home);
        if Path::new(&final_path).exists() {
            Some(final_path)
        } else {
            warn!(
                "Config file not found in `{}`. Loading defaults...",
                final_path
            );
            None
        }
    } else {
        warn!("Could not resolve `$HOME`. Loading default config...");
        None
    }
}

/*
 * Default config values:
 */
fn text_color() -> String {
    "#FFFFFF".to_string()
}
fn background_color() -> String {
    "#111416".to_string()
}
fn reader_show_bottom_indicator() -> bool {
    true
}
