use slint::{ComponentHandle, SharedString, Weak, Model, ModelRc, VecModel};
use yt_dlp::cache::playlist;
use std::sync::Arc;

use crate::App;
use crate::YoutubeDownloader;
use crate::config::config::Config;

pub struct Playlist {
    app: Weak<App>,
    youtube_downloader: Arc<YoutubeDownloader>,
}

impl Playlist {
    pub fn new(app: &App, youtube_downloader: Arc<YoutubeDownloader>) -> Self {
        Self {
            app: app.as_weak(),
            youtube_downloader,
        }
    }

    pub async fn manage_playlist(&self) {
        if let Some(app) = self.app.upgrade() {
            app.on_add_playlist({
                // let youtube_downloader = Arc::clone(&self.youtube_downloader);
                let config: Config = Config::load().await.unwrap_or_default();
                let app_weak = app.as_weak();
                move |playlist_name: SharedString, playlist_url: SharedString| {
                    let mut config = config.clone();
                    let playlist_for_config = playlist_url.clone().to_string();
                    tokio::spawn(async move {
                        // let youtube_downloader = Arc::clone(&youtube_downloader);

                        let _ = config
                            .update_playlist(playlist_url.to_string(), playlist_for_config)
                            .await;
                    });

                    if let Some(app) = app_weak.upgrade() {
                        let playlists: slint::ModelRc<SharedString> = app.get_playlists();
                        let mut playlists_vec: Vec<SharedString> = playlists.iter().collect::<Vec<_>>();
                        playlists_vec.push(SharedString::from(playlist_name.clone()));
                        app.set_playlists(ModelRc::new(VecModel::from(playlists_vec)));
                    }
                }
            });
        }
    }
}
