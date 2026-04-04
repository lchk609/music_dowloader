use std::{path::PathBuf, process::{Command, Output}};

use crate::dowloaders::dowloader_base::DownloaderBase;

pub struct OtherMediaDownloader {
    pub downloader_base: DownloaderBase,
}

#[derive(serde::Deserialize, Debug)]
struct AudioInfo {
    title: String,
    codec: String,
    url: String,
}

impl OtherMediaDownloader {
    pub fn new(downloader_base: DownloaderBase) -> Self {
        Self { downloader_base }
    }

    pub async fn fetch_music_infos(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output: Output = Command::new("yt-dlp")
            .arg("-f ba")
            .arg("--print-json")
            .arg("-x")
            .arg("--audio-format opus")
            .arg("-embed-metadata --embed-thumbnail --convert-thumbnails jpg")
            .arg(url)
            .output()
            .expect("Failed to execute yt-dlp command");

        let output_str = String::from_utf8_lossy(&output.stdout);

        let audio_info: AudioInfo =
            serde_json::from_str(&output_str).expect("Failed to parse JSON output");

        Ok(())
    }

    pub async fn download_music(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output_dir = match &self.downloader_base.output_dir {
            Some(dir) => dir.clone(),
            None => self
                .downloader_base
                .config
                .lock()
                .await
                .saved_directory
                .clone()
                .unwrap_or_else(|| std::path::PathBuf::from("output")),
        };

        let music_output: String =
            match output_dir.join("%(uploader)s - %(title)s.%(ext)s").to_str() {
                Some(path) => path.to_string(),
                None => {
                    eprintln!("Failed to convert output path to string");
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid output path",
                    )));
                }
            };

        println!("Downloading music to: {}", music_output);

        let output = Command::new(self.downloader_base.libraries.youtube.to_str().unwrap())
            .args([
                "-f",
                "ba",
                "-o",
                &music_output,
                "--print",
                "after_move:\"%(title)s|%(filepath)s\"",
                url,
            ])
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);

        let parts: Vec<&str> = stdout.trim().split('|').collect();

        let title = parts.get(0).unwrap_or(&"");
        let path = parts.get(1).unwrap_or(&"");

        println!("Titre: {}", title);
        println!("Path: {}", path);

        Ok(())
    }

    async fn convert_music(&self, codec: &str, input_path: PathBuf, output_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {

        
        Ok(())
    }
}
