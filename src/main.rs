use std::sync::Arc;

use dowloaders::music::MusicDownloader;

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
    let app: Arc<App>  = Arc::new(App::new().unwrap());
    app.window().set_size(slint::PhysicalSize::new(800, 600));
    setup::setup_gui(Arc::clone(&app), downloader_base).await?;

    app.run().unwrap();

    Ok(())
}
