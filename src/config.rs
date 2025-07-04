use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmoduleConfig {
    pub name: String,
    pub path: PathBuf, // Path within the monorepo
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppConfig {
    pub submodules: Vec<SubmoduleConfig>,
}

fn get_config_path(config_dir: &Path) -> PathBuf {
    config_dir.join(CONFIG_FILE_NAME)
}

pub fn load_or_create_config(config_dir: &Path) -> io::Result<AppConfig> {
    let config_path = get_config_path(config_dir);
    if config_path.exists() {
        let mut file = fs::File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(AppConfig::default())
    }
}

pub fn save_config(config_dir: &Path, config: &AppConfig) -> io::Result<()> {
    let config_path = get_config_path(config_dir);
    let contents = serde_json::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut file = fs::File::create(config_path)?;
    file.write_all(contents.as_bytes())
}
