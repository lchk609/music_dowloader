use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use directories::ProjectDirs;

use crate::enums::codec::CODEC_PREFERENCE;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub saved_directory: Option<PathBuf>,
    pub playlists: HashMap<String, String>,
    // key = playlist id
    // value = playlist name
    pub codec: CODEC_PREFERENCE,
}

fn config_path() -> std::path::PathBuf {
    let proj = ProjectDirs::from("", "", "RustPlaylistApp")
        .expect("Could not determine config directory");

    let dir = proj.config_dir();
    std::fs::create_dir_all(dir).expect("Failed to create config directory");

    dir.join("config.json")
}

impl Config {
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path();
        save_config(self, &path).await
    }

    pub async fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_path();
        load_config(&path).await
    }
}

async fn save_config(config: &Config, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config).unwrap();
    fs::write(path, json)
        .await
        .map_err(|e| format!("Failed to save config: {}", e).into())
}

async fn load_config(path: &std::path::Path) -> Result<Config, Box<dyn std::error::Error>> {
    if path.exists() {
        let json = fs::read_to_string(path).await?;
        let config: Config = serde_json::from_str(&json)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}
