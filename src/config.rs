use std::default::Default;
use std::path::{Path, PathBuf};

use crate::keybind;

#[derive(Debug, thiserror::Error)]
pub enum ConfigReadError {
    #[error("could not read config file: {0}")]
    IoError(std::io::Error),
    #[error("could not deserialize config file contents: {0}")]
    DeserializeError(ron::error::Error),
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ConfigDirectives {
    #[serde(rename = "live-reload")]
    pub live_reload_configuration: bool,
}

impl Default for ConfigDirectives {
    fn default() -> Self {
        ConfigDirectives {
            live_reload_configuration: false,
        }
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Metric {
    Percent(f32),
    Absolute(f32),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Push {
        direction: Direction,
        fraction: f32,
    },
    Nudge {
        direction: Direction,
        distance: Metric,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Binding {
    pub key: keybind::Key,
    pub modifiers: Vec<keybind::Modifier>,
    pub action: Action,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub directives: ConfigDirectives,
    pub bindings: Vec<Binding>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            directives: ConfigDirectives::default(),
            bindings: vec![
                Binding {
                    key: keybind::Key::Left,
                    modifiers: vec![keybind::Modifier::Super, keybind::Modifier::Shift],
                    action: Action::Push {
                        direction: Direction::Left,
                        fraction: 0.5,
                    },
                },
                Binding {
                    key: keybind::Key::Left,
                    modifiers: vec![keybind::Modifier::Super, keybind::Modifier::Shift],
                    action: Action::Nudge {
                        direction: Direction::Left,
                        distance: Metric::Absolute(100.0),
                    },
                },
            ],
        }
    }
}

pub fn read_config_from_file(path: &dyn AsRef<Path>) -> Result<Option<Config>, ConfigReadError> {
    if !path.as_ref().exists() {
        return Ok(None);
    }

    let config_string = std::fs::read_to_string(path).map_err(|e| ConfigReadError::IoError(e))?;
    ron::from_str(&config_string).map_err(|e| ConfigReadError::DeserializeError(e))
}

pub fn default_config_path() -> PathBuf {
    let mut cfg_dir = dirs::config_dir().expect("Could not find user configuration directory.");
    cfg_dir.push("whimsy");
    cfg_dir.push("whimsy.ron");
    cfg_dir
}

pub fn create_default_config() -> std::io::Result<()> {
    let default_config = Config::default();
    let default_path = default_config_path();
    // This should always succeed; the default config should always be representable.
    let config_string =
        ron::ser::to_string_pretty(&default_config, ron::ser::PrettyConfig::default()).unwrap();
    std::fs::create_dir_all(&default_path.parent().unwrap())?;
    std::fs::write(&default_path, &config_string)?;
    Ok(())
}
