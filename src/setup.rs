use slint::Model;
use std::path::PathBuf;
use std::sync::Arc;
use slint::{ModelRc, SharedString, VecModel};
use crate::dowloaders::youtube::YoutubeDownloader;
use crate::ui::components::download_button;
use crate::{App, Song};
use crate::config::config::{Config};

async fn load_music_on_opening(app: &App, output_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries: tokio::fs::ReadDir = tokio::fs::read_dir(output_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "mp3" || extension == "flac" || extension == "wav" {
                    let title: String = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    println!("Loading music file: {}", title);
                    let mut songs: Vec<Song> = app.get_songs().iter().collect();
                    songs.push(Song {
                        title: SharedString::from(title),
                        download_id: 0,
                        total: 100,
                        download_progress: 100,
                    });
                    app.set_songs(ModelRc::new(VecModel::from(songs)));
                }
            }
        }
    }

    Ok(())
}

async fn setup_event_listiners(app: &App, youtube_downloader: Arc<YoutubeDownloader>) -> Result<(), Box<dyn std::error::Error>> {
    download_button::manage_add_music(app, youtube_downloader).await;
    Ok(())
}

pub async fn setup_gui(app: &App, youtube_downloader: Arc<YoutubeDownloader>) -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = Config::load().await?;
    
    let music_path: PathBuf = match config.saved_directory.clone() {
        Some(saved_directory) => saved_directory,
        None => {
            eprintln!("Impossible de trouver le répertoire utilisateur.");
            return Err("Impossible de trouver le répertoire utilisateur.".into());
        }
    };
    load_music_on_opening(app, music_path).await?;
    setup_event_listiners(app, youtube_downloader).await?;

    Ok(())
}

pub async fn setup_dowloader() -> Result<YoutubeDownloader, Box<dyn std::error::Error>> {
    let config: Config = Config::load().await?;
    
    let outpout_dir: PathBuf = match config.saved_directory.clone() {
        Some(saved_directory) => saved_directory,
        None => {
            eprintln!("Impossible de trouver le répertoire utilisateur.");
            return Err("Impossible de trouver le répertoire utilisateur.".into());
        }
    };

    let youtube_downloader: YoutubeDownloader = YoutubeDownloader::new(
        outpout_dir,
        crate::enums::codec::CODEC_PREFERENCE::MP3,
        3,
    );
    youtube_downloader.dowload_tools().await?;
    Ok(youtube_downloader)
}
