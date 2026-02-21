use dowloaders::youtube::YoutubeDownloader;
use std::sync::Arc;

mod dowloaders;
mod setup;
mod ui;
mod config;
mod enums;
slint::include_modules!();

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let youtube_downloader: YoutubeDownloader = setup::setup_dowloader().await?;
    let app: App = App::new().unwrap();
    setup::setup_gui(&app, Arc::new(youtube_downloader)).await?;

    app.run().unwrap();

    Ok(())
}
