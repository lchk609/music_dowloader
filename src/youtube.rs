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

pub async fn download_audio_stream_from_url(url: String, codec: CODEC_PREFERENCE, custom_output_dir: String) -> Result<(), Box<dyn std::error::Error>> {
    let libraries_dir: PathBuf = PathBuf::from("libs");

    let output_dir: PathBuf = get_or_create_output_dir(custom_output_dir).unwrap().to_path_buf();

    let youtube: PathBuf = libraries_dir.join("yt-dlp");
    let ffmpeg: PathBuf = libraries_dir.join("ffmpeg");

    println!("Output directory: {}", &output_dir.display());

    let libraries = Libraries::new(youtube, ffmpeg);
    let fetcher = Youtube::new(libraries, output_dir).await?;


    let video_infos: yt_dlp::prelude::Video = fetcher.fetch_video_infos(url).await?;

    let video_title = format!("{}.m4a", video_infos.title);

    println!("Downloading audio for video: {}", &video_title);

    fetcher.download_audio_stream(&video_infos, &video_title).await?;

    if codec != CODEC_PREFERENCE::AAC {

        let codec_str = match codec {
            CODEC_PREFERENCE::FLAC => "flac",
            CODEC_PREFERENCE::MP3 => "mp3",
            CODEC_PREFERENCE::AAC => "aac",
            CODEC_PREFERENCE::WAV => "wav",
            _ => "mp3",
        };

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
        .status()?;

    if !status.success() {
        return Err(format!("Conversion en {} échouée : {}", codec_str, status).into());
    }

    std::fs::remove_file(input_file)?;
    }

    Ok(())
}

fn get_or_create_output_dir(mut path: String) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let user_home = match dirs::home_dir() {
        Some(path_home) => path_home,
        None => {
            eprintln!("Impossible de trouver le répertoire utilisateur.");
            return Err("Impossible de trouver le répertoire utilisateur.".into());
        }
    };

    println!("User home directory: {:?}", &user_home);
    if path.is_empty() {
        path = String::from("MusicDL");
    }
    let output_dir = PathBuf::from(path);
    if !output_dir.exists() {
        std::fs::create_dir_all(user_home.join(&output_dir))?;
    }
    Ok(user_home.join(&output_dir))
}
