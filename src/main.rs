use std::io::{self, BufRead};
use std::sync::Arc;
use slint::{ModelRc, VecModel};
use tokio::sync::mpsc;
use dowloaders::youtube::{YoutubeDownloader};

use crate::models::Song;

mod dowloaders;
mod enums;
mod models;

slint::include_modules!();

// #[tokio::main]
// pub async fn main() -> Result<(), Box<dyn std::error::Error>> {

//     let app = App::new().unwrap();
//     app.run().unwrap();
    
//     let youtube_downloader: Arc<YoutubeDownloader> = Arc::new(YoutubeDownloader::new(std::path::PathBuf::new(), enums::codec::CODEC_PREFERENCE::MP3, 3));
//     youtube_downloader.dowload_tools().await?;

//     let (tx, mut rx) = mpsc::channel::<String>(32);

//     let input_task = tokio::spawn(async move {
//         loop {
//             let url = read_line_from_stdin().await?;
//             if url.eq_ignore_ascii_case("exit") {
//                 tx.send("exit".to_string()).await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
//                 break;
//             }
//             tx.send(url).await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
//         }
//         Ok::<(), io::Error>(())
//     });

//     while let Some(url) = rx.recv().await {
//         if url == "exit" {
//             break;
//         }
//         let youtube_downloader: Arc<YoutubeDownloader> = Arc::clone(&youtube_downloader);
//         tokio::spawn(async move {
//             match youtube_downloader.download_audio_stream_from_url(&url).await {
//                 Ok(_) => println!("Téléchargement terminé : {}", url),
//                 Err(e) => eprintln!("Erreur pour {} : {}", url, e),
//             }
//         });
//     }

//     let _ = input_task.await?;

//     Ok(())
// }

// async fn read_line_from_stdin() -> io::Result<String> {
//     println!("Entrez l'URL de la musique à télécharger (ou 'exit') :");
//     let stdin = io::BufReader::new(io::stdin());
//     let mut lines = stdin.lines();
//     let url = lines.next().unwrap().unwrap_or_default();
//     Ok(url.trim().to_string())
// }

fn main() {
    let app = App::new().unwrap();

    // Create a dynamic model
    let songs_model = Song {
        title: "title".into(),
        downloading: true
    };

    app.set_songs(ModelRc::from(songs_model.clone()));

    // When download button is clicked
    app.on_download_clicked(move || {
        songs_model.push(songs_model);
    });

    app.run().unwrap();
}
