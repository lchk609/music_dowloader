use crate::enums::codec::CodecPreference;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::Semaphore;
use yt_dlp::Youtube;
use yt_dlp::client::deps::Libraries;

pub struct YoutubeDownloader {
    output_dir: PathBuf,
    codec_preference: CodecPreference,
    semaphore: Arc<Semaphore>,
    libraries: Arc<Libraries>,
}

impl YoutubeDownloader {
    pub fn new(
        output_dir: PathBuf,
        codec_preference: CodecPreference,
        max_concurrent: usize,
    ) -> Self {
        let libraries_dir = PathBuf::from("libs");

        let youtube = libraries_dir.join("yt-dlp");
        let ffmpeg = libraries_dir.join("ffmpeg");

        let libraries = Libraries::new(youtube, ffmpeg);
        Self {
            output_dir,
            codec_preference,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            libraries: Arc::new(libraries),
        }
    }

    pub async fn dowload_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        let executables_dir = PathBuf::from("libs");
        let output_dir = PathBuf::from("output");

        Youtube::with_new_binaries(executables_dir, output_dir).await?;
        Ok(())
    }

    pub async fn dowload_audio_stream_with_progress(
        &self,
        url: &String,
        callback: impl Fn(&str) -> Box<dyn Fn(u64, u64) + Send + 'static + std::marker::Sync>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let _permit = self.semaphore.acquire().await?;

        let fetcher = Youtube::new((*self.libraries).clone(), self.output_dir.clone()).await?;

        let video_infos: yt_dlp::prelude::Video = fetcher.fetch_video_infos(url.clone()).await?;

        let video_title = format!("{}.mp4", video_infos.title);

        let codec_str = &self.codec_preference.to_string();

        if self
            .check_if_music_already_exists(&video_infos.title, &fetcher.output_dir, &codec_str)
            .await
        {
            println!(
                "Le fichier {} existe déjà dans le répertoire de sortie. Téléchargement ignoré.",
                &video_infos.title
            );
            return Ok(Default::default());
        }

        println!("Downloading audio for video: {}", &video_title);

        let download_id = fetcher
            .download_video_with_progress(&video_infos, &video_title, callback(&video_infos.title))
            .await?;

        fetcher.wait_for_download(download_id).await;

        self.transform_video_to_audio(&video_infos.title).await?;

        Ok(video_infos.title)
    }

    async fn transform_video_to_audio(
        &self,
        video_title: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let codec_str = &self.codec_preference.to_string();
        let video_path = self.output_dir.join(format!("{}.mp4", video_title));
        println!("Converting video to audio for: {}", video_path.display());
        let output_path = self
            .output_dir
            .join(format!("{}.{}", video_title, codec_str));
        println!("Output path for audio: {}", output_path.display());
        let status: std::process::ExitStatus = Command::new(&self.libraries.ffmpeg)
            .arg("-i")
            .arg(&video_path)
            .arg("-c:a")
            .arg(codec_str)
            .arg("-compression_level")
            .arg("5")
            .arg(output_path)
            .status()
            .await?;

        if !status.success() {
            return Err(format!("Conversion en {} échouée : {}", codec_str, status).into());
        }

        fs::remove_file(video_path).await?;

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
}
