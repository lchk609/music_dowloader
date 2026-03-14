use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::App;
use crate::config::config::Config;
use crate::dowloaders::dowloader_base::DownloaderBase;
use crate::dowloaders::playlist::PlaylistDownloader;
use crate::events::download_events::CustomDownloadEvent;

pub struct Playlist {
    app: Weak<App>,
    playlist_downloader: Arc<PlaylistDownloader>,
    tx: Arc<UnboundedSender<CustomDownloadEvent>>,
}

impl Playlist {
    pub fn new(
        app: &App,
        downloader_base: DownloaderBase,
        tx: Arc<UnboundedSender<CustomDownloadEvent>>,
    ) -> Self {
        let playlist_downloader = Arc::new(PlaylistDownloader::new(downloader_base));
        Self {
            app: app.as_weak(),
            playlist_downloader,
            tx,
        }
    }

    pub async fn manage_playlist(&self) {
        if let Some(app) = self.app.upgrade() {
            let tx = self.tx.clone();
            app.on_add_playlist({
                let playlist_downloader = Arc::clone(&self.playlist_downloader);
                let config: Config = Config::load().await.unwrap_or_default();
                let app_weak = app.as_weak();
                move |playlist_name: SharedString, playlist_url: SharedString| {
                    let mut config = config.clone();
                    let playlist_name_for_config = playlist_name.clone().to_string();
                    let playlist_url_for_config = playlist_url.clone().to_string();
                    let playlist_downloader = Arc::clone(&playlist_downloader);
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let _ = config
                            .update_playlist(playlist_url_for_config, playlist_name_for_config)
                            .await;
                    });

                    if let Some(app) = app_weak.upgrade() {
                        let playlists: slint::ModelRc<SharedString> = app.get_playlists();
                        let mut playlists_vec: Vec<SharedString> =
                            playlists.iter().collect::<Vec<_>>();
                        playlists_vec.push(SharedString::from(playlist_name.clone()));
                        app.set_playlists(ModelRc::new(VecModel::from(playlists_vec)));
                    }

                    tokio::spawn(async move {
                        let _ = playlist_downloader
                            .download_playlist(
                                &playlist_url.to_string(),
                                playlist_name.to_string().as_str(),
                                tx.clone(),
                            )
                            .await;
                    });
                }
            });
        }
    }
}
