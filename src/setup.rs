use crate::config::config::Config;
use crate::dowloaders::dowloader_base::DownloaderBase;
use crate::dowloaders::music::MusicDownloader;
use crate::events::download_events::CustomDownloadEvent;
use crate::ui::components::song_item::ItemManagement;
use crate::ui::components::{download_button, playlist};
use crate::{App, Playlist, Song, Settings};
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{Mutex, Semaphore};
use tokio::sync::mpsc::unbounded_channel;
use yt_dlp::client::Libraries;

struct MusicFile {
    title: String,
    date_added: std::time::SystemTime,
}

async fn load_music_on_opening(
    app: Arc<App>,
    output_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut song_files: Vec<MusicFile> = match collect_music_files(output_dir).await {
        Ok(songs) => songs,
        Err(_) => Vec::new(),
    };

    song_files.sort_by(|a, b| b.date_added.cmp(&a.date_added));

    let mut songs: Vec<Song> = app.get_songs().iter().collect();

    songs.extend(song_files.into_iter().map(|music_file| Song {
        title: SharedString::from(music_file.title),
        is_downloading: false,
        download_id: SharedString::from(String::new()),
    }));

    app.set_songs(ModelRc::new(VecModel::from(songs)));

    Ok(())
}

async fn collect_music_files(dir: PathBuf) -> Result<Vec<MusicFile>, Box<dyn std::error::Error>> {
    let mut song_files = Vec::new();
    let mut stack = VecDeque::new();
    stack.push_back(dir);

    while let Some(current_dir) = stack.pop_front() {
        let mut entries = fs::read_dir(&current_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push_back(path);
            } else if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "mp3" || extension == "flac" || extension == "wav" {
                        let title = path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let date_added = entry
                            .metadata()
                            .await?
                            .created()
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        song_files.push(MusicFile { title, date_added });
                    }
                }
            }
        }
    }

    Ok(song_files)
}

async fn load_playlists_on_opening(
    app: Arc<App>,
    config: Arc<Config>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut playlists: Vec<Playlist> = Vec::new();

    for playlist_info in config.playlists.iter() {
        playlists.push(Playlist {
            id: playlist_info.id.to_shared_string(),
            title: playlist_info.name.to_shared_string(),   
        });
    }

    app.set_playlists(ModelRc::new(VecModel::from(playlists)));

    Ok(())
}

async fn setup_event_listiners(
    app: Arc<App>,
    downloader_base: DownloaderBase,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = unbounded_channel::<CustomDownloadEvent>();

    ItemManagement::new(rx).start_listening(Arc::clone(&app));

    let tx_arc = Arc::new(tx);

    let download_button =
        download_button::DownloadButton::new(Arc::clone(&app), downloader_base.clone(), Arc::clone(&tx_arc));
    download_button.manage_add_music().await;
    let playlist = playlist::Playlists::new(Arc::clone(&app), downloader_base.clone(), Arc::clone(&tx_arc)).await;
    playlist.manage_playlist().await;
    Ok(())
}

pub async fn setup_gui(
    app: Arc<App>,
    downloader_base: DownloaderBase,
) -> Result<(), Box<dyn std::error::Error>> {
    let config:Arc<Config>  = Arc::new(Config::load().await?);

    let music_path: PathBuf = match config.saved_directory.clone() {
        Some(saved_directory) => saved_directory,
        None => PathBuf::new(),
    };

    get_or_create_output_dir(music_path.to_string_lossy().to_string(), Arc::clone(&config)).await?;
    load_config_in_ui(Arc::clone(&app), Arc::clone(&config)).await?;

    load_music_on_opening(Arc::clone(&app), music_path).await?;
    load_playlists_on_opening(Arc::clone(&app), Arc::clone(&config)).await?;
    setup_event_listiners(Arc::clone(&app), downloader_base).await?;

    Ok(())
}

async fn load_config_in_ui(app: Arc<App>, config: Arc<Config>) -> Result<(), Box<dyn std::error::Error>> {
    let Some(save_directory) = config.saved_directory.clone() else {
        return Ok(());
    };
    let settings: Settings = Settings {
        save_directory: save_directory.to_string_lossy().to_string().to_shared_string(),
        codec: config.codec.clone().to_string().to_shared_string(),
        max_concurrent_download: config.max_concurrent_downloads,
    };

    app.set_settings(settings.into());

    Ok(())
}

pub async fn setup_dowloader() -> Result<DownloaderBase, Box<dyn std::error::Error>> {
    let config: Arc<Config> = Arc::new(Config::load().await?);

    let output_dir: PathBuf = match config.saved_directory.clone() {
        Some(saved_directory) => saved_directory,
        None => PathBuf::new(),
    };

    get_or_create_output_dir(output_dir.to_string_lossy().to_string(), Arc::clone(&config)).await?;

    let libraries_dir = PathBuf::from("libs");

    let youtube = libraries_dir.join("yt-dlp");
    let ffmpeg = libraries_dir.join("ffmpeg");

    let libraries = Libraries::new(youtube, ffmpeg);

    let downlader_base = DownloaderBase {
        libraries,
        codec_preference: config.codec.clone(),
        output_dir,
        semaphore: Arc::new(Semaphore::new(config.max_concurrent_downloads as usize)),
        config: Arc::new(Mutex::new(Config::load().await.unwrap_or_default()))
    };

    let music_downloader: MusicDownloader = MusicDownloader::new(downlader_base.clone());
    music_downloader.download_tools().await?;
    Ok(downlader_base)
}

async fn get_or_create_output_dir(
    mut path: String,
    config: Arc<Config>,
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
        ..(*config).clone()
    }
    .save()
    .await?;

    Ok(output_dir)
}
