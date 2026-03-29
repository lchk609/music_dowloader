use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel, Weak};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use tokio::sync::Mutex;

use crate::config::config::Config;
use crate::dowloaders::dowloader_base::DownloaderBase;
use crate::dowloaders::playlist::PlaylistDownloader;
use crate::events::download_events::CustomDownloadEvent;
use crate::{App, AppLogic, Playlist};

pub struct Playlists {
    app: Arc<App>,
    playlist_downloader: Arc<PlaylistDownloader>,
    tx: Arc<UnboundedSender<CustomDownloadEvent>>,
    config: Arc<Mutex<Config>>
}

impl Playlists {
    pub async fn new(
        app: Arc<App>,
        downloader_base: DownloaderBase,
        tx: Arc<UnboundedSender<CustomDownloadEvent>>,
    ) -> Self {
        let playlist_downloader: Arc<PlaylistDownloader> = Arc::new(PlaylistDownloader::new(downloader_base));
        Self {
            app: app,
            playlist_downloader,
            tx,
            config: Arc::new(Mutex::new(Config::load().await.unwrap_or_default()))
        }
    }

    pub async fn manage_playlist(&self) {
        if let Some(app) = self.app.as_weak().upgrade() {
            let tx: Arc<UnboundedSender<CustomDownloadEvent>> = Arc::clone(&self.tx);
            app.global::<AppLogic>().on_add_playlist({
                let playlist_downloader = Arc::clone(&self.playlist_downloader);
                let app: Weak<App> = app.as_weak();
                let shared_config: Arc<Mutex<Config>> = Arc::clone(&self.config);
                move |playlist_name: SharedString, playlist_url: SharedString| {
                    let playlist_name_for_config: String = playlist_name.clone().to_string();
                    let playlist_url_for_config: String = playlist_url.clone().to_string();
                    let playlist_downloader: Arc<PlaylistDownloader> = Arc::clone(&playlist_downloader);
                    let tx: Arc<UnboundedSender<CustomDownloadEvent>> = Arc::clone(&tx);
                    let app: Weak<App> = app.clone();
                    let shared_config: Arc<Mutex<Config>> = Arc::clone(&shared_config);
                    tokio::spawn({
                        let shared_config: Arc<Mutex<Config>> = Arc::clone(&shared_config);
                        async move {
                        let mut config: tokio::sync::MutexGuard<'_, Config>  = shared_config.lock().await;  
                        match config
                            .update_playlist(
                                &playlist_url_for_config,
                                &playlist_name_for_config,
                            )
                            .await
                        {
                            Ok(uuid) => {
                                let playlist: Playlist = Playlist {
                                    id: uuid.to_shared_string(),
                                    title: SharedString::from(playlist_name_for_config.clone()),
                                };
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(app) = app.upgrade() {
                                        manage_playlist_layout(app.as_weak(), playlist, true);
                                    } else {
                                        println!("app déjà détruite");
                                    }
                                });
                            }
                            Err(err) => println!("{}", err),
                        }
                    }});

                    tokio::spawn(async move {
                        let config: tokio::sync::MutexGuard<'_, Config>  = shared_config.lock().await;
                        let _ = playlist_downloader
                            .download_playlist(
                                &playlist_url.to_string(),
                                playlist_name.to_string().as_str(),
                                Arc::clone(&tx),
                                config.max_concurrent_downloads
                            )
                            .await;
                    });
                }
            });

            app.global::<AppLogic>().on_refresh_playlist({
                let playlist_downloader = Arc::clone(&self.playlist_downloader);
                let tx = Arc::clone(&self.tx);
                move |playlist_id: SharedString| {
                    println!("playlist to refresh : {}", playlist_id);
                    let playlist_downloader = Arc::clone(&playlist_downloader);
                    let tx = Arc::clone(&tx);
                    tokio::spawn(
                        async move {
                        if let Ok(uuid) = Uuid::from_str(playlist_id.as_str()) {
                            let _ = playlist_downloader.refresh_playlist(uuid, Arc::clone(&tx)).await;
                        } else {
                            println!("playlist id : {}", playlist_id);
                            println!("Can't refresh the playlist");
                        }
                    });
                }
            });

            app.global::<AppLogic>().on_delete_playlist({
                let app: App = app.clone_strong();
                let shared_config: Arc<Mutex<Config>> = Arc::clone(&self.config);
                move |playlist_id: SharedString| {
                    let app_weak = app.as_weak();
                    let shared_config: Arc<Mutex<Config>> = Arc::clone(&shared_config);
                    tokio::spawn({
                        let shared_config = Arc::clone(&shared_config);
                        let playlist_id = playlist_id.clone();
                        async move {
                        let mut config: tokio::sync::MutexGuard<'_, Config>  = shared_config.lock().await;
                        if let Ok(uuid) = Uuid::from_str(playlist_id.as_str()) {
                            match config.remove_playlist(uuid).await {
                                Ok(playlist_id) => {
                                    let _ = slint::invoke_from_event_loop(move || {
                                        if let Some(app) = app_weak.upgrade() {
                                            let playlists: ModelRc<Playlist> = app.get_playlists();
                                            let playlist: Playlist = playlists
                                                .iter()
                                                .find(|item| {
                                                    item.id == playlist_id.to_shared_string()
                                                })
                                                .unwrap();
                                            manage_playlist_layout(app.as_weak(), playlist, false);
                                        } else {
                                            println!("app déjà détruite");
                                        }
                                    });
                                }
                                Err(err) => println!("{:?}", err),
                            };
                        } else {
                            println!("playlist id : {}", playlist_id);
                            println!("Can't remove the playlist");
                        }
                    }});
                }
            })
        }
    }
}

fn manage_playlist_layout(app: Weak<App>, playlist: Playlist, add_playlist: bool) {
    let app_clone = app.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(app) = app_clone.upgrade() {
            if add_playlist {
                let playlists: ModelRc<crate::Playlist> = app.get_playlists();
                let mut playlists_vec: Vec<crate::Playlist> = playlists.iter().collect::<Vec<_>>();
                playlists_vec.push(playlist);
                app.set_playlists(ModelRc::new(VecModel::from(playlists_vec)));
            } else {
                let playlists: ModelRc<crate::Playlist> = app.get_playlists();
                let mut playlists_vec: Vec<crate::Playlist> = playlists.iter().collect::<Vec<_>>();
                playlists_vec.retain(|item| item.id != playlist.id);
                app.set_playlists(ModelRc::new(VecModel::from(playlists_vec)));
            }
        }
    });
}
