use std::{path::PathBuf, sync::Arc};

use futures::stream::{self, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;
use yt_dlp::{Downloader, model::playlist::Playlist};

use crate::{
    dowloaders::{dowloader_base::DownloaderBase, music::MusicDownloader},
    events::download_events::CustomDownloadEvent,
};

#[derive(Clone, Debug)]
pub struct PlaylistDownloader {
    downloader_base: DownloaderBase,
}

impl PlaylistDownloader {
    pub fn new(downloader_base: DownloaderBase) -> Self {
        Self { downloader_base }
    }

    pub async fn download_playlist(
        &self,
        playlist_url: &str,
        playlist_name: &str,
        event_tx: Arc<mpsc::UnboundedSender<CustomDownloadEvent>>,
        concurrency: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let downloader = Downloader::builder(
            self.downloader_base.libraries.clone(),
            self.downloader_base.config.lock().await.saved_directory.clone().unwrap_or_else(|| PathBuf::from("output")),
        )
        .build()
        .await?;

        println!("Fetching playlist infos for URL: {}", playlist_url);

        let playlist_infos: Playlist = downloader.fetch_playlist_infos(playlist_url).await?;

        let output_dir: PathBuf = self.downloader_base.config.lock().await.saved_directory.clone().unwrap_or_else(|| PathBuf::from("output")).join(playlist_name);
        let downloader_base: DownloaderBase = self.downloader_base.clone();
        let new_dowloader_base: DownloaderBase = DownloaderBase {
            output_dir,
            ..downloader_base
        };

        let downloader: Arc<MusicDownloader> =
            Arc::new(MusicDownloader::new(new_dowloader_base.clone()));

        stream::iter(playlist_infos.entries)
            .map(|video| {
                let downloader = Arc::clone(&downloader);
                let tx = Arc::clone(&event_tx);
                async move {
                    if let Err(err) = downloader
                        .download_audio_stream_with_hooks(&video.url, tx)
                        .await
                    {
                        eprintln!("download failed: {:?}", err);
                    }
                }
            })
            .buffer_unordered(concurrency as usize)
            .collect::<Vec<_>>()
            .await;

        Ok(())
    }

    pub async fn refresh_playlist(
        &self,
        playlist_id: Uuid,
        event_tx: Arc<mpsc::UnboundedSender<CustomDownloadEvent>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config: tokio::sync::MutexGuard<'_, crate::config::config::Config> =
            self.downloader_base.config.lock().await;

        if let Some(playlist) = config.playlists.iter().find(|item| item.id == playlist_id) {
            self.download_playlist(
                &playlist.url,
                &playlist.name,
                event_tx,
                config.max_concurrent_downloads,
            )
            .await?;
        };

        Ok(())
    }
}
