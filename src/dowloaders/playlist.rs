use std::{path::PathBuf, sync::Arc};

use threadpool::ThreadPool;
use tokio::sync::{Semaphore, mpsc};
use yt_dlp::{Downloader, model::playlist::Playlist};

use crate::{
    dowloaders::{dowloader_base::DownloaderBase, youtube::YoutubeDownloader}, events::download_events::CustomDownloadEvent,
};

#[derive(Clone, Debug)]
pub struct PlaylistDownloader {
    downloader_base: DownloaderBase,
    semaphore: Arc<Semaphore>,
}

impl PlaylistDownloader {
    pub fn new(downloader_base: DownloaderBase) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(downloader_base.max_concurrent)),
            downloader_base,
        }
    }

    pub async fn download_playlist(
        &self,
        playlist_url: &str,
        playlist_name: &str,
        event_tx: mpsc::UnboundedSender<CustomDownloadEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let downloader = Downloader::builder(
            self.downloader_base.libraries.clone(),
            self.downloader_base.output_dir.clone(),
        )
        .build()
        .await?;

        println!("Fetching playlist infos for URL: {}", playlist_url);

        let playlist_infos: Playlist = downloader.fetch_playlist_infos(playlist_url).await?;

        println!("playlist fetched");


        let output_dir = self.downloader_base.output_dir.clone().join(playlist_name);
        let downloader_base = self.downloader_base.clone();
        let semaphore = Arc::new(Semaphore::new(4));
        let new_dowloader_base: DownloaderBase = DownloaderBase {
            output_dir,
            .. downloader_base
        };

        for video in playlist_infos.entries {
            let tx = event_tx.clone();

            let downloader:YoutubeDownloader  = YoutubeDownloader::new(new_dowloader_base.clone());
            downloader
            .download_audio_stream_with_hooks(&video.url, tx)
                .await?;
        }

        Ok(())
    }
}
