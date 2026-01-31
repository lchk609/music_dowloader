use std::io::{self, BufRead, Write};
use tokio::sync::mpsc;

mod dowloaders;
mod enums;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let youtube_downloader: dowloaders::youtube::YoutubeDownloader = dowloaders::youtube::YoutubeDownloader::new(std::path::PathBuf::new(), enums::codec::CODEC_PREFERENCE::MP3);
    youtube_downloader.dowload_tools().await?;

    let (tx, mut rx) = mpsc::channel::<String>(32);

    let input_task = tokio::spawn(async move {
        loop {
            let url = read_line_from_stdin().await.unwrap();
            if url.eq_ignore_ascii_case("exit") {
                tx.send("exit".to_string()).await.unwrap();
                break;
            }
            tx.send(url).await.unwrap();
        }
    });

    while let Some(url) = rx.recv().await {
        if url == "exit" {
            break;
        }
        match youtube_downloader.download_audio_stream_from_url(url).await {
            Ok(_) => println!("Téléchargement terminé avec succès !"),
            Err(e) => eprintln!("Erreur : {}", e),
        }
    }

    input_task.await?;

    Ok(())
}

async fn read_line_from_stdin() -> io::Result<String> {
    println!("Entrez l'URL de la musique à télécharger (ou 'quitter') :");
    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    let url = lines.next().unwrap().unwrap_or_default();
    Ok(url.trim().to_string())
}
