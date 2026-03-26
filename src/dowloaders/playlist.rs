use std::{path::PathBuf, sync::Arc};

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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let downloader = Downloader::builder(
            self.downloader_base.libraries.clone(),
            self.downloader_base.output_dir.clone(),
        )
        .build()
        .await?;

        println!("Fetching playlist infos for URL: {}", playlist_url);

        let playlist_infos: Playlist = downloader.fetch_playlist_infos(playlist_url).await?;

        let output_dir: PathBuf = self.downloader_base.output_dir.clone().join(playlist_name);
        let downloader_base: DownloaderBase = self.downloader_base.clone();
        let new_dowloader_base: DownloaderBase = DownloaderBase {
            output_dir,
            ..downloader_base
        };

        let downloader: Arc<MusicDownloader> =
            Arc::new(MusicDownloader::new(new_dowloader_base.clone()));

        for video in playlist_infos.entries {
            let downloader_clone = Arc::clone(&downloader);
            let tx_clone = Arc::clone(&event_tx);
            let _ = self.downloader_base.semaphore.acquire().await;
            tokio::spawn(async move {
                if let Err(err) = downloader_clone
                    .download_audio_stream_with_hooks(&video.url, tx_clone)
                    .await
                {
                    eprintln!("download failed, err : {:?}", err);
                }
            });
        }

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
            self.download_playlist(&playlist.url, &playlist.name, event_tx)
                .await?;
        };

        Ok(())
    }
}
