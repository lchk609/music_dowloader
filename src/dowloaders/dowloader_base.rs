use std::{path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, Semaphore};
use yt_dlp::client::Libraries;

use crate::{config::config::Config};

#[derive(Clone, Debug)]
pub struct DownloaderBase {
    pub libraries: Libraries,
    pub output_dir: PathBuf,
    pub semaphore: Arc<Semaphore>,
    pub config: Arc<Mutex<Config>>,
}