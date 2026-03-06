use slint::{ComponentHandle, SharedString, Weak};
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
                move |playlist_name: SharedString, playlist_url: SharedString| {
                    let mut config = config.clone();
                    tokio::spawn(async move {
                        // let youtube_downloader = Arc::clone(&youtube_downloader);

                        let _ = config
                            .update_playlist(playlist_url.to_string(), playlist_name.to_string())
                            .await;

                        println!("Add playlist button clicked");
                        println!(
                            "Playlist name: {}, Playlist URL: {}",
                            playlist_name, playlist_url
                        );
                    });
                }
            });
        }
    }
}
