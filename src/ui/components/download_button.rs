use crate::App;
use crate::AppLogic;
use crate::MusicDownloader;
use crate::dowloaders::dowloader_base::DownloaderBase;
use crate::events::download_events::CustomDownloadEvent;
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub struct DownloadButton {
    app: Weak<App>,
    music_downloader: Arc<MusicDownloader>,
    tx: Arc<UnboundedSender<CustomDownloadEvent>>,
}

impl DownloadButton {
    pub fn new(
        app: Arc<App>,
        downloader_base: DownloaderBase,
        tx: Arc<UnboundedSender<CustomDownloadEvent>>,
    ) -> Self {
        let music_downloader = Arc::new(MusicDownloader::new(downloader_base));

        Self {
            app: app.as_weak(),
            music_downloader,
            tx,
        }
    }

    pub async fn manage_add_music(&self) {
        if let Some(app) = self.app.upgrade() {
            app.global::<AppLogic>().on_download_clicked({
                let music_downloader = Arc::clone(&self.music_downloader);
                let tx = self.tx.clone();
                move |url: SharedString| {
                    let music_downloader = Arc::clone(&music_downloader);
                    let url = url.to_string();
                    println!("Download button clicked");

                    let tx = tx.clone();
                    tokio::spawn(async move {
                        music_downloader
                            .download_audio_stream_with_hooks(&url, tx)
                            .await
                            .unwrap();
                    });
                }
            });
        }
    }
}
