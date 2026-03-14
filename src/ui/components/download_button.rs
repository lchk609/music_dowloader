use crate::App;
use crate::YoutubeDownloader;
use crate::dowloaders::dowloader_base::DownloaderBase;
use crate::events::download_events::CustomDownloadEvent;
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub struct DownloadButton {
    app: Weak<App>,
    youtube_downloader: Arc<YoutubeDownloader>,
    tx: Arc<UnboundedSender<CustomDownloadEvent>>,
}

impl DownloadButton {
    pub fn new(
        app: &App,
        downloader_base: DownloaderBase,
        tx: Arc<UnboundedSender<CustomDownloadEvent>>,
    ) -> Self {
        let youtube_downloader = Arc::new(YoutubeDownloader::new(downloader_base));

        Self {
            app: app.as_weak(),
            youtube_downloader,
            tx,
        }
    }

    pub async fn manage_add_music(&self) {
        if let Some(app) = self.app.upgrade() {
            app.on_download_clicked({
                let youtube_downloader = Arc::clone(&self.youtube_downloader);
                let tx = self.tx.clone();
                move |url: SharedString| {
                    let youtube_downloader = Arc::clone(&youtube_downloader);
                    let url = url.to_string();
                    println!("Download button clicked");

                    let tx = tx.clone();
                    tokio::spawn(async move {
                        youtube_downloader
                            .download_audio_stream_with_hooks(&url, tx)
                            .await
                            .unwrap();
                    });
                }
            });
        }
    }
}
