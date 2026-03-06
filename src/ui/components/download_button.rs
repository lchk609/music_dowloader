use crate::App;
use crate::Song;
use crate::YoutubeDownloader;
use crate::events::download_events::CustomDownloadEvent;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use yt_dlp::events::DownloadEvent;

pub struct DownloadButton {
    app: Weak<App>,
    youtube_downloader: Arc<YoutubeDownloader>,
}

impl DownloadButton {
    pub fn new(app: &App, youtube_downloader: Arc<YoutubeDownloader>) -> Self {
        Self {
            app: app.as_weak(),
            youtube_downloader,
        }
    }

    pub async fn manage_add_music(&self) {
        if let Some(app) = self.app.upgrade() {
            let (tx, rx) = unbounded_channel::<CustomDownloadEvent>();

            ItemManagement::new(rx).start_listening(app.as_weak());

            app.on_download_clicked({
                let youtube_downloader = Arc::clone(&self.youtube_downloader);
                let tx = tx.clone();
                move |url: SharedString| {
                    let youtube_downloader = Arc::clone(&youtube_downloader);
                    let url = url.to_string();
                    println!("Download button clicked");

                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let title = youtube_downloader
                            .download_audio_stream_with_hooks(&url, tx)
                            .await
                            .unwrap();

                        println!("Download finished: {}", title);
                    });
                }
            });
        }
    }
}

struct ItemManagement {
    rx: UnboundedReceiver<CustomDownloadEvent>,
}

impl ItemManagement {
    pub fn new(rx: UnboundedReceiver<CustomDownloadEvent>) -> Self {
        Self { rx }
    }

    pub fn start_listening(mut self, app: Weak<App>) {
        tokio::spawn(async move {
            while let Some(event) = self.rx.recv().await {
                let app_clone = app.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_clone.upgrade() {
                        match event.download_event {
                            DownloadEvent::DownloadStarted { .. } => {
                                println!("Download started for: {}", event.music_title);
                                let mut songs: Vec<Song> = app.get_songs().iter().collect();
                                songs.push(Song {
                                    title: event.music_title.clone().into(),
                                    is_downloading: true,
                                    download_id: SharedString::from(event.music_title.clone()),
                                });
                                app.set_songs(ModelRc::new(VecModel::from(songs)));
                            }
                            DownloadEvent::DownloadCompleted { .. } => {
                                println!("Download completed for: {}", event.music_title);
                                let mut songs: Vec<Song> = app.get_songs().iter().collect();
                                if let Some(song) = songs
                                    .iter_mut()
                                    .find(|s| s.download_id == event.music_title)
                                {
                                    song.is_downloading = false;
                                    song.download_id = SharedString::new();
                                    app.set_songs(ModelRc::new(VecModel::from(songs)));
                                }
                            }
                            DownloadEvent::DownloadFailed { .. } => {
                                //manage dowload failure, e.g. show a message to the user
                                println!("Download failed for: {}", event.music_title);
                            }
                            DownloadEvent::DownloadProgress { .. } => {
                                let mut songs: Vec<Song> = app.get_songs().iter().collect();
                                let exists = songs.iter().any(|s| {
                                    s.download_id
                                        == event.download_event.download_id().unwrap().to_string()
                                        || s.title == event.music_title
                                });
                                if !exists {
                                    songs.push(Song {
                                        title: event.music_title.clone().into(),
                                        is_downloading: true,
                                        download_id: SharedString::from(event.music_title.clone()),
                                    });
                                    app.set_songs(ModelRc::new(VecModel::from(songs)));
                                }
                            }
                            _ => {}
                        }
                    }
                })
                .unwrap();
            }
        });
    }
}
