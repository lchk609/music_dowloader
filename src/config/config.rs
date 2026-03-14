use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::enums::codec::CodecPreference;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Config {
    pub saved_directory: Option<PathBuf>,
    pub playlists: Vec<PlaylistInfo>,
    pub codec: CodecPreference,
    pub max_concurrent_downloads: usize,
}

#[derive(Serialize, Debug, Deserialize, Clone, Default)]
pub struct PlaylistInfo {
    pub url: String,
    pub name: String,
    #[serde(default)]
    pub last_updated: DateTime<Utc>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            saved_directory: None,
            playlists: Vec::new(),
            codec: CodecPreference::default(),
            max_concurrent_downloads: 5,
        }
    }
}

async fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let proj =
        ProjectDirs::from("", "", "MusicDownloader").expect("Could not determine config directory");

    let dir = proj.config_dir();
    fs::create_dir_all(dir).await?;

    Ok(dir.join("config.json"))
}

impl Config {
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path().await?;
        save_config(self, &path).await
    }

    pub async fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_path().await?;
        load_config(&path).await
    }

    pub async fn update_playlist(
        &mut self,
        playlist_url: String,
        playlist_name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.playlists.push(PlaylistInfo {
            url: playlist_url,
            name: playlist_name,
            last_updated: Utc::now(),
        });
        self.save().await
    }
}

async fn save_config(
    config: &Config,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json: String = serde_json::to_string_pretty(config).unwrap();
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
