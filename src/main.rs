use dowloaders::youtube::YoutubeDownloader;
use std::sync::Arc;

use crate::dowloaders::dowloader_base::DownloaderBase;

mod config;
mod dowloaders;
mod enums;
mod events;
mod setup;
mod ui;
slint::include_modules!();

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(debug_assertions, feature = "tracing"))]
    {
        tracing_subscriber::fmt::init();
    }
    let downloader_base: DownloaderBase = setup::setup_dowloader().await?;
    let app: App = App::new().unwrap();
    setup::setup_gui(&app, downloader_base).await?;

    app.run().unwrap();

    Ok(())
}
