use yt_dlp::events::DownloadEvent;

#[derive(Debug, Clone)]
pub struct CustomDownloadEvent {
    pub download_event: DownloadEvent,
    pub music_title: String,
}