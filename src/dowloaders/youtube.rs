use crate::dowloaders::dowloader_base::DownloaderBase;
use crate::enums::codec::CodecPreference;
use crate::events::download_events::CustomDownloadEvent;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use yt_dlp::Downloader;
use yt_dlp::client::deps::Libraries;
use yt_dlp::events::{DownloadEvent, EventFilter, EventHook, HookResult};
use yt_dlp::model::playlist::Playlist;

pub struct YoutubeDownloader {
    downloader_base: DownloaderBase,
    semaphore: Arc<Semaphore>,
}

impl YoutubeDownloader {
    pub fn new(downloader_base: DownloaderBase) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(downloader_base.max_concurrent)),
            downloader_base,
        }
    }

    pub async fn download_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        let executables_dir = PathBuf::from("libs");
        let output_dir = PathBuf::from("output");

        Downloader::with_new_binaries(executables_dir, output_dir)
            .await?
            .build()
            .await?;

        Ok(())
    }

    async fn check_if_music_already_exists(
        &self,
        title: &str,
        output_dir: &PathBuf,
        codec: &str,
    ) -> bool {
        let path = output_dir.join(format!("{}.{}", title, codec));
        fs::metadata(&path).await.is_ok()
    }

    pub async fn download_audio_stream_with_hooks(
        &self,
        url: &str,
        event_tx: mpsc::UnboundedSender<CustomDownloadEvent>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let _permit = self.semaphore.acquire().await?;

        let mut downloader = Downloader::builder(self.downloader_base.libraries.clone(), self.downloader_base.output_dir.clone())
            .build()
            .await?;

        let video_infos = downloader.fetch_video_infos(url).await?;

        println!("Video title: {}", video_infos.title);

        if self
            .check_if_music_already_exists(
                &video_infos.title,
                &self.downloader_base.output_dir,
                &self.downloader_base.codec_preference.to_string(),
            )
            .await
        {
            println!("Music already exists: {}", &video_infos.title);
            return Ok(video_infos.title);
        }

        let hook = MusicDownloadEvent::new(event_tx.clone(), video_infos.title.clone());
        downloader.register_hook(hook.clone()).await;

        let downloader = Arc::new(downloader);
        let downloader_for_events = Arc::clone(&downloader);
        let hook_for_events = hook.clone();

        tokio::spawn(async move {
            let mut rx = downloader_for_events.subscribe_events();
            while let Ok(event) = rx.recv().await {
                if let Err(e) = hook_for_events.on_event(&event).await {
                    eprintln!("Hook execution error: {:?}", e);
                }
            }
        });

        let output_path = format!("{}.webm", video_infos.title);

        let _ = downloader
            .download(&video_infos, output_path.clone())
            .audio_quality(yt_dlp::model::AudioQuality::Best)
            .audio_codec(self.downloader_base.codec_preference.to_yt_dlp_codec())
            .execute_audio_stream()
            .await;

        self.convert_audio(&video_infos.title.as_str()).await?;

        println!("Download completed for URL: {}", url);

        Ok(video_infos.title)
    }

    async fn convert_audio(&self, audio_title: &str) -> Result<(), String> {
        let input = self.downloader_base.output_dir.join(format!("{}.webm", audio_title));
        let output = self.downloader_base.output_dir.join(format!(
            "{}.{}",
            audio_title,
            self.downloader_base.codec_preference.to_string().to_lowercase()
        ));

        println!("Converting audio from {:?} to {:?}", input, output);
        let ffmpeg_args = match self.downloader_base.codec_preference {
            CodecPreference::MP3 => vec![
                "-i",
                input.to_str().unwrap(),
                "-f",
                "mp3",
                "-acodec",
                "libmp3lame",
                "-q:a",
                "2", // Qualité 0-9 (2 = haute qualité)
                output.to_str().unwrap(),
            ],
            CodecPreference::OGG => vec![
                "-i",
                input.to_str().unwrap(),
                "-f",
                "ogg",
                "-acodec",
                "libvorbis",
                "-q:a",
                "6", // Qualité 0-10 (6 = bonne qualité)
                output.to_str().unwrap(),
            ],
            CodecPreference::FLAC => vec![
                "-i",
                input.to_str().unwrap(),
                "-f",
                "flac",
                "-acodec",
                "flac",
                output.to_str().unwrap(),
            ],
            CodecPreference::WAV => vec![
                "-i",
                input.to_str().unwrap(),
                "-f",
                "wav",
                "-acodec",
                "pcm_s16le",
                output.to_str().unwrap(),
            ],
            CodecPreference::AAC => {
                vec![
                    "-i",
                    input.to_str().unwrap(),
                    "-f",
                    "adts",
                    "-acodec",
                    "aac",
                    "-b:a",
                    "192k", // Bitrate de 192 kbps
                    output.to_str().unwrap(),
                ]
            }
        };

        let status = Command::new(&self.downloader_base.libraries.ffmpeg)
            .args(&ffmpeg_args)
            .status()
            .await
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err(format!("FFmpeg conversion failed: {:?}", status));
        }

        fs::remove_file(input)
            .await
            .map_err(|e| format!("Failed to remove temporary file: {}", e))?;

        Ok(())
    }
}

#[derive(Clone)]
struct MusicDownloadEvent {
    tx: mpsc::UnboundedSender<CustomDownloadEvent>,
    music_title: String,
}

impl MusicDownloadEvent {
    fn new(tx: mpsc::UnboundedSender<CustomDownloadEvent>, music_title: String) -> Self {
        Self { tx, music_title }
    }
}

#[async_trait]
impl EventHook for MusicDownloadEvent {
    async fn on_event(&self, event: &DownloadEvent) -> HookResult {
        let _ = self.tx.send(CustomDownloadEvent {
            download_event: event.clone(),
            music_title: self.music_title.clone(),
        });

        Ok(())
    }

    fn filter(&self) -> EventFilter {
        EventFilter::all()
    }
}
