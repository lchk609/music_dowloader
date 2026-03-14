use std::{path::PathBuf};
use std::marker::Copy;
use yt_dlp::client::Libraries;

use crate::enums::codec::CodecPreference;

#[derive(Clone, Debug)]
pub struct DownloaderBase {
    pub libraries: Libraries,
    pub codec_preference: CodecPreference,
    pub output_dir: PathBuf,
    pub max_concurrent: usize,
}