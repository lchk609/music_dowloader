use yt_dlp::{Youtube};
use yt_dlp::client::deps::Libraries;
use std::path::PathBuf;
use tokio::process::Command;
use crate::enums::codec::{self, CODEC_PREFERENCE};
use tokio::fs;
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct YoutubeDownloader {
    output_dir: PathBuf,
    codec_preference: CODEC_PREFERENCE,
    semaphore: Arc<Semaphore>,
    libraries: Arc<Libraries>,
}

pub struct VideoInfo {
    pub title: String,
    pub url: String,
}

impl YoutubeDownloader {    
    pub fn new(output_dir: PathBuf, codec_preference: CODEC_PREFERENCE, max_concurrent: usize) -> Self {

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
    
    pub async fn download_audio_stream_from_url(&self, url: &String) -> Result<(), Box<dyn std::error::Error>> {
        let _permit = self.semaphore.acquire().await?;
    
        let output_dir: PathBuf = get_or_create_output_dir(self.output_dir.to_string_lossy().to_string()).await?;
            
        let fetcher = Youtube::new((*self.libraries).clone(), output_dir).await?;
    
    
        let video_infos: yt_dlp::prelude::Video = fetcher.fetch_video_infos(url.clone()).await?;
    
        let video_title = format!("{}.m4a", video_infos.title);
    
        let codec_str = codec::get_codec_extension(&self.codec_preference);
    
        if check_if_music_already_exists(&video_infos.title, &fetcher.output_dir, &codec_str).await {
            println!("Le fichier {} existe déjà dans le répertoire de sortie. Téléchargement ignoré.", &video_infos.title);
            return Ok(());
        }
    
        println!("Downloading audio for video: {}", &video_title);
    
        fetcher.download_audio_stream(&video_infos, &video_title).await?;
    
        if self.codec_preference != CODEC_PREFERENCE::AAC {
    
            let input_file = fetcher.output_dir.join(&video_title);
            let output_file = fetcher.output_dir.join(format!("{}.{}", video_infos.title, codec_str));
                let status: std::process::ExitStatus = Command::new(fetcher.libraries.ffmpeg)
            .arg("-i")
            .arg(&input_file)
            .arg("-c:a")
            .arg(codec_str)
            .arg("-compression_level")
            .arg("5")
            .arg(&output_file)
            .status()
            .await?;
    
        if !status.success() {
            return Err(format!("Conversion en {} échouée : {}", codec_str, status).into());
        }
    
        fs::remove_file(input_file).await?;
        }
    
        Ok(())
    }

    pub async fn get_videos_infos(&self, url: &String) -> Result<VideoInfo, Box<dyn std::error::Error>> {
        let output_dir: PathBuf = get_or_create_output_dir(self.output_dir.to_string_lossy().to_string()).await?;

        let fetcher: Youtube = Youtube::new((*self.libraries).clone(), output_dir).await?;
    
    
        let video_infos: yt_dlp::prelude::Video = fetcher.fetch_video_infos(url.clone()).await?;
    
        Ok(VideoInfo {
            title: video_infos.title,
            url: url.clone(),
        })
    }
}


async fn get_or_create_output_dir(mut path: String) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let user_home = match dirs::home_dir() {
        Some(path_home) => path_home,
        None => {
            eprintln!("Impossible de trouver le répertoire utilisateur.");
            return Err("Impossible de trouver le répertoire utilisateur.".into());
        }
    };

    if path.is_empty() {
        path = String::from("MusicDL");
    }

    let output_dir = user_home.join(&path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).await?;
    }

    Ok(output_dir)
}

async fn check_if_music_already_exists(title: &str, output_dir: &PathBuf, codec: &str) -> bool {
    let path = output_dir.join(format!("{}.{}", title, codec));
    fs::metadata(&path).await.is_ok()
}
