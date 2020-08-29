use std::default::Default;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ConfigReadError {
    IoError(std::io::Error),
    DeserializeError(toml::de::Error),
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ConfigDirectives {
    #[serde(rename = "live_reload")]
    pub live_reload_configuration: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Keybind {}

impl Default for ConfigDirectives {
    fn default() -> Self {
        ConfigDirectives {
            live_reload_configuration: false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub directives: ConfigDirectives,
}

pub fn read_config_from_file(path: &dyn AsRef<Path>) -> Result<Config, ConfigReadError> {
    let config_string = std::fs::read_to_string(path).map_err(|e| ConfigReadError::IoError(e))?;
    toml::from_str(&config_string).map_err(|e| ConfigReadError::DeserializeError(e))
}

pub fn default_config_path() -> PathBuf {
    let mut cfg_dir = dirs::config_dir().expect("Could not find user configuration directory.");
    cfg_dir.push("whimsy");
    cfg_dir.push("whimsy.toml");
    cfg_dir
}

pub fn create_default_config() -> std::io::Result<()> {
    let default_config = Config::default();
    let default_path = default_config_path();
    // This should always succeed; the default config should always be representable.
    let config_string = toml::to_string_pretty(&default_config).unwrap();
    std::fs::create_dir_all(&default_path.parent().unwrap())?;
    std::fs::write(&default_path, &config_string)?;
    Ok(())
}
