use crate::config::config::Config;
use crate::dowloaders::youtube::YoutubeDownloader;
use crate::ui::components::download_button;
use crate::{App, Song};
use slint::Model;
use slint::{ModelRc, SharedString, VecModel};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

struct MusicFile {
    title: String,
    date_added: std::time::SystemTime,
}

async fn load_music_on_opening(
    app: &App,
    output_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries: tokio::fs::ReadDir = tokio::fs::read_dir(output_dir).await?;

    let mut song_files: Vec<MusicFile> = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "mp3" || extension == "flac" || extension == "wav" {
                    let title: String = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let date_added = entry.metadata().await?.created().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    song_files.push(MusicFile { title, date_added });
                }
            }
        }
    }

    song_files.sort_by(|a, b| b.date_added.cmp(&a.date_added));

    let mut songs: Vec<Song> = app.get_songs().iter().collect();

    songs.extend(song_files.into_iter().map(|music_file| Song {
        title: SharedString::from(music_file.title),
    }));

    app.set_songs(ModelRc::new(VecModel::from(songs)));

    Ok(())
}

async fn setup_event_listiners(
    app: &App,
    youtube_downloader: Arc<YoutubeDownloader>,
) -> Result<(), Box<dyn std::error::Error>> {
    download_button::manage_add_music(app, youtube_downloader).await;
    Ok(())
}

pub async fn setup_gui(
    app: &App,
    youtube_downloader: Arc<YoutubeDownloader>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = Config::load().await?;

    let music_path: PathBuf = match config.saved_directory.clone() {
        Some(saved_directory) => saved_directory,
        None => PathBuf::new(),
    };

    get_or_create_output_dir(music_path.to_string_lossy().to_string(), config).await?;

    load_music_on_opening(app, music_path).await?;
    setup_event_listiners(app, youtube_downloader).await?;

    Ok(())
}

pub async fn setup_dowloader() -> Result<YoutubeDownloader, Box<dyn std::error::Error>> {
    let config: Config = Config::load().await?;

    let output_dir: PathBuf = match config.saved_directory.clone() {
        Some(saved_directory) => saved_directory,
        None => PathBuf::new(),
    };

    get_or_create_output_dir(output_dir.to_string_lossy().to_string(), config).await?;

    let youtube_downloader: YoutubeDownloader =
        YoutubeDownloader::new(output_dir, crate::enums::codec::CODEC_PREFERENCE::MP3, 3);
    youtube_downloader.dowload_tools().await?;
    Ok(youtube_downloader)
}

async fn get_or_create_output_dir(
    mut path: String,
    config: Config,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let user_home: PathBuf = match directories::UserDirs::new() {
        Some(path_home) => path_home.home_dir().to_path_buf(),
        None => {
            eprintln!("Impossible de trouver le répertoire utilisateur.");
            return Err("Impossible de trouver le répertoire utilisateur.".into());
        }
    };

    if path.is_empty() {
        path = String::from("MusicDL");
    }

    let output_dir = user_home.join(&path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).await?;
    }

    Config {
        saved_directory: Some(output_dir.clone()),
        ..config
    }
    .save()
    .await?;

    Ok(output_dir)
}
