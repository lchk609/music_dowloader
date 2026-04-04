use crate::App;
use crate::AppLogic;
use crate::MusicDownloader;
use crate::dowloaders::dowloader_base::DownloaderBase;
use crate::dowloaders::other_media;
use crate::dowloaders::other_media::OtherMediaDownloader;
use crate::events::download_events::CustomDownloadEvent;
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub struct DownloadButton {
    app: Weak<App>,
    music_downloader: Arc<MusicDownloader>,
    other_media_downloader: Arc<OtherMediaDownloader>,
    tx: Arc<UnboundedSender<CustomDownloadEvent>>,
}

impl DownloadButton {
    pub fn new(
        app: Arc<App>,
        downloader_base: DownloaderBase,
        tx: Arc<UnboundedSender<CustomDownloadEvent>>,
    ) -> Self {
        let music_downloader = Arc::new(MusicDownloader::new(downloader_base.clone()));
        let other_media_downloader = Arc::new(OtherMediaDownloader::new(downloader_base.clone()));

        Self {
            app: app.as_weak(),
            music_downloader,
            other_media_downloader,
            tx,
        }
    }

    pub async fn manage_add_music(&self) {
        if let Some(app) = self.app.upgrade() {
            app.global::<AppLogic>().on_download_clicked({
                // let music_downloader = Arc::clone(&self.music_downloader);
                let other_media_downloader = Arc::clone(&self.other_media_downloader);
                let tx = self.tx.clone();
                move |url: SharedString| {
                    // let music_downloader = Arc::clone(&music_downloader);
                    let other_media_downloader = Arc::clone(&other_media_downloader);
                    let url = url.to_string();
                    println!("Download button clicked");

                    let tx = tx.clone();
                    tokio::spawn(async move {
                        other_media_downloader.download_music(&url).await.unwrap();
                        // music_downloader
                        //     .download_audio_stream_with_hooks(&url, tx)
                        //     .await
                        //     .unwrap();
                    });
                }
            });
        }
    }
}
