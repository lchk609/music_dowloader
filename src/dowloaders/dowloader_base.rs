use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;
use yt_dlp::client::Libraries;

use crate::enums::codec::CodecPreference;

#[derive(Clone, Debug)]
pub struct DownloaderBase {
    pub libraries: Libraries,
    pub codec_preference: CodecPreference,
    pub output_dir: PathBuf,
    pub semaphore: Arc<Semaphore>,
}