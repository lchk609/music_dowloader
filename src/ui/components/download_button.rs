use crate::App;
use crate::Song;
use crate::YoutubeDownloader;
use slint::ModelRc;
use slint::{ComponentHandle, Model, SharedString, VecModel};
use yt_dlp::download;
use std::sync::Arc;

pub struct DownloadButton {
    app: Arc<App>,
    youtube_downloader: Arc<YoutubeDownloader>,
}

impl DownloadButton {
    pub fn new(app: &App, youtube_downloader: Arc<YoutubeDownloader>) -> Self {
        Self {
            app: Arc::new(app.clone_strong()),
            youtube_downloader,
        }
    }

    pub async fn manage_add_music(&self) {
        self.app.on_download_clicked({
            let youtube_downloader = Arc::clone(&self.youtube_downloader);
            move |url: SharedString| {
                let youtube_downloader = Arc::clone(&youtube_downloader);
                let url = url.to_string();
                println!("Download button clicked");

                tokio::spawn(async move {
                    let youtube_downloader = Arc::clone(&youtube_downloader);
                    youtube_downloader
                        .download_audio_stream_with_hooks(&url)
                        .await
                        .unwrap();
                });
            }
        })
    }

    pub async fn add_song_item(&self, download_id: &str, title: &str) {
        let mut songs: Vec<Song> = self.app.get_songs().iter().collect();
        songs.push(Song {
            title: title.into(),
            is_downloading: true,
            download_id: SharedString::from(download_id),
        });
        self.app.set_songs(ModelRc::new(VecModel::from(songs)));
    }

    pub async fn set_is_dowloading(&self, download_id: &str, is_downloading: bool) {
        let mut songs: Vec<Song> = self.app.get_songs().iter().collect();
        if let Some(song) = songs.iter_mut().find(|s| s.download_id == download_id) {
            song.is_downloading = is_downloading;
            self.app.set_songs(ModelRc::new(VecModel::from(songs)));
        }
    }

    pub async fn remove_dowload_id_from_song_item(&self, download_id: &str) {
        let mut songs: Vec<Song> = self.app.get_songs().iter().collect();
        if let Some(song) = songs.iter_mut().find(|s| s.download_id == download_id) {
            song.download_id = SharedString::new();
            self.app.set_songs(ModelRc::new(VecModel::from(songs)));
        }
    }
}
