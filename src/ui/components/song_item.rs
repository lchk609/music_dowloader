use slint::{Model, ModelRc, SharedString, VecModel, Weak};
use tokio::sync::mpsc::UnboundedReceiver;
use yt_dlp::events::DownloadEvent;

use crate::{App, Song, events::download_events::CustomDownloadEvent};

pub struct ItemManagement {
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
                                songs.insert(0, Song {
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
                                    songs.insert(0, Song {
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
