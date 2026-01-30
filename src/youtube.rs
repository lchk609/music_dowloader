use yt_dlp::{Youtube};
use yt_dlp::client::deps::Libraries;
use std::path::PathBuf;
use std::process::Command;
use crate::codec::{CODEC_PREFERENCE};

pub async fn dowload_tools() -> Result<(), Box<dyn std::error::Error>> {
    let executables_dir = PathBuf::from("libs");
    let output_dir = PathBuf::from("output");

    Youtube::with_new_binaries(executables_dir, output_dir).await?;
    Ok(())
}

pub async fn download_audio_stream_from_url(url: String, codec: CODEC_PREFERENCE) -> Result<(), Box<dyn std::error::Error>> {
    let libraries_dir: PathBuf = PathBuf::from("libs");
    let output_dir: PathBuf = PathBuf::from("output");

    let youtube: PathBuf = libraries_dir.join("yt-dlp");
    let ffmpeg: PathBuf = libraries_dir.join("ffmpeg");

    // println!("Downloading audio stream from URL: {}", url);

    let libraries = Libraries::new(youtube, ffmpeg);
    let fetcher = Youtube::new(libraries, output_dir).await?;

    let video_infos: yt_dlp::prelude::Video = fetcher.fetch_video_infos(url).await?;

    let video_title = format!("{}.m4a", video_infos.title);
    let codec_str = match codec {
        CODEC_PREFERENCE::FLAC => "flac",
        CODEC_PREFERENCE::MP3 => "mp3",
        CODEC_PREFERENCE::AAC => "aac",
        CODEC_PREFERENCE::WAV => "wav",
    };

    println!("Downloading audio for video: {}", &video_title);

    fetcher.download_audio_stream(&video_infos, &video_title).await?;

    if codec != CODEC_PREFERENCE::AAC {
        let input_file = fetcher.output_dir.join(&video_title);
        let output_file = fetcher.output_dir.join(format!("{}.{}", video_infos.title, codec_str));
            let status = Command::new(fetcher.libraries.ffmpeg)
        .arg("-i")
        .arg(&input_file)
        .arg("-c:a")
        .arg(codec_str)
        .arg("-compression_level")
        .arg("5")
        .arg(&output_file)
        .status()?;

    if !status.success() {
        return Err(format!("Conversion en {} échouée : {}", codec_str, status).into());
    }

    // 3. Nettoyer
    std::fs::remove_file(input_file)?;
    }

    Ok(())
}
