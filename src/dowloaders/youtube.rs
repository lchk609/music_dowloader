use crate::dowloaders;
use crate::enums::codec::{self, CodecPreference};
use async_trait::async_trait;
use futures::stream::StreamExt;
use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};
use yt_dlp::Downloader;
use yt_dlp::client::deps::Libraries;
use yt_dlp::events::{DownloadEvent, EventFilter, EventHook, HookResult};
use yt_dlp::model::playlist::{Playlist, PlaylistDownloadProgress};

pub struct YoutubeDownloader {
    output_dir: PathBuf,
    codec_preference: CodecPreference,
    // semaphore: Arc<Semaphore>,
    libraries: Libraries,
}

impl YoutubeDownloader {
    pub async fn new(
        output_dir: PathBuf,
        codec_preference: CodecPreference,
        max_concurrent: usize,
    ) -> Self {
        let libraries_dir = PathBuf::from("libs");

        let youtube = libraries_dir.join("yt-dlp");
        let ffmpeg = libraries_dir.join("ffmpeg");

        let libraries = Libraries::new(youtube, ffmpeg);

        Self {
            output_dir: output_dir.clone(),
            codec_preference,
            // semaphore: Arc::new(Semaphore::new(max_concurrent)),
            libraries: libraries.clone(),
        }
    }

    pub async fn download_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        let executables_dir = PathBuf::from("libs");
        let output_dir = PathBuf::from("output");

        // Create fetcher and install binaries
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
    ) -> Result<String, Box<dyn std::error::Error>> {
        println!("Starting download for URL: {}", url);

        let mut downloader = Downloader::builder(self.libraries.clone(), self.output_dir.clone())
            .build()
            .await?;

        let hook: MusicDownloadEvent = MusicDownloadEvent;
        downloader.register_hook(hook.clone()).await;

        // Wrap the downloader in Arc to share across async tasks
        let downloader = Arc::new(downloader);
        let downloader_for_events = Arc::clone(&downloader);
        let hook_for_events = hook.clone();

        // Subscribe to event bus in a background task and manually execute hook
        // Since hook_registry is private in yt-dlp, we subscribe directly
        tokio::spawn(async move {
            let mut rx = downloader_for_events.subscribe_events();
            while let Ok(event) = rx.recv().await {
                // Call the hook's on_event method directly for each event received
                if let Err(e) = hook_for_events.on_event(&event).await {
                    eprintln!("Hook execution error: {:?}", e);
                }
            }
        });

        let video_infos = downloader.fetch_video_infos(url).await?;

        println!("Video title: {}", video_infos.title);

        if self
            .check_if_music_already_exists(
                &video_infos.title,
                &self.output_dir,
                &self.codec_preference.to_string(),
            )
            .await
        {
            println!("Music already exists: {}", &video_infos.title);
            return Ok(video_infos.title);
        }

        let output_path = format!("{}.webm", video_infos.title);

        let _ = downloader
            .download(&video_infos, output_path.clone())
            .audio_quality(yt_dlp::model::AudioQuality::Best)
            .audio_codec(self.codec_preference.to_yt_dlp_codec())
            .execute_audio_stream()
            .await;

        self.convert_audio(&video_infos.title.as_str()).await?;

        println!("Download completed for URL: {}", url);

        Ok(video_infos.title)
    }

    async fn convert_audio(&self, audio_title: &str) -> Result<(), String> {
        let input = self.output_dir.join(format!("{}.webm", audio_title));
        let output = self.output_dir.join(format!(
            "{}.{}",
            audio_title,
            self.codec_preference.to_string().to_lowercase()
        ));

        println!("Converting audio from {:?} to {:?}", input, output);
        let ffmpeg_args = match self.codec_preference {
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

        let status = Command::new(&self.libraries.ffmpeg)
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
struct MusicDownloadEvent;

#[async_trait]
impl EventHook for MusicDownloadEvent {
    async fn on_event(&self, event: &DownloadEvent) -> HookResult {
        match event {
            DownloadEvent::DownloadStarted {
                download_id,
                format_id,
                ..
            } => {
                println!(
                    "Download {} started with format: {}",
                    download_id,
                    format_id.as_deref().unwrap_or("unknown")
                );
            }
            DownloadEvent::DownloadCompleted {
                download_id,
                output_path,
                ..
            } => {
                println!("Download {} completed: {:?}", download_id, output_path);
            }
            DownloadEvent::DownloadProgress {
                download_id,
                downloaded_bytes,
                total_bytes,
                speed_bytes_per_sec,
                eta_seconds,
            } => {
                let progress = format!(
                    "{:.2}%",
                    (*downloaded_bytes as f64 / *total_bytes as f64) * 100.0
                );
                println!(
                    "Download {} progress: {} ({} / {:?} bytes, speed: {:?} B/s, ETA: {:?} seconds)",
                    download_id,
                    progress,
                    downloaded_bytes,
                    total_bytes,
                    speed_bytes_per_sec,
                    eta_seconds
                );
            }
            DownloadEvent::DownloadFailed {
                download_id, error, ..
            } => {
                eprintln!("Download {} failed: {}", download_id, error);
            }
            _ => {
                println!("Other event: {:?}", event.event_type());
            }
        }
        Ok(())
    }

    fn filter(&self) -> EventFilter {
        // Only receive terminal events (completed, failed, canceled)
        EventFilter::all()
    }
}
