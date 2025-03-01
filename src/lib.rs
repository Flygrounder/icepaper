use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub background: Option<String>,
    pub gallery: Vec<String>,
}

pub fn get_config_path() -> Option<PathBuf> {
    xdg::BaseDirectories::new()
        .ok()?
        .place_config_file(Path::new("icepaper/config.toml"))
        .ok()
}

pub fn read_config() -> Option<Config> {
    let content = fs::read_to_string(get_config_path().unwrap()).ok()?;
    toml::from_str::<Config>(&content).ok()
}

pub fn write_config(config: &Config) -> Option<()> {
    let config_contents = toml::to_string(&config).unwrap();
    fs::write(get_config_path().unwrap(), config_contents).ok()
}
