use std::io::{self, BufRead};
use std::sync::Arc;
use slint::{Model, ModelRc, VecModel, SharedString};
use tokio::sync::mpsc;
use dowloaders::youtube::{YoutubeDownloader, VideoInfo};

mod dowloaders;
mod enums;

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

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let youtube_downloader: Arc<YoutubeDownloader> = Arc::new(YoutubeDownloader::new(std::path::PathBuf::new(), enums::codec::CODEC_PREFERENCE::MP3, 3));
    youtube_downloader.dowload_tools().await?;
    let app = App::new().unwrap();
    let app_handle = app.as_weak();

    // Create a dynamic model
    let songs_model: Vec<Song> = vec![
        Song {
            title: "title".into(),
            downloading: true,
        },
    ];

    app.set_songs(ModelRc::new(VecModel::from(songs_model)));

    app.on_download_clicked(async move |url: SharedString| {
        let url = url.to_string();
        let youtube_downloader: Arc<YoutubeDownloader> = Arc::clone(&youtube_downloader);

        tokio::spawn(async move {
            match youtube_downloader.download_audio_stream_from_url(&url).await {
                Ok(_) => println!("Téléchargement terminé : {}", url),
                Err(e) => eprintln!("Erreur pour {} : {}", url, e),
            }
        });

        let app: App = app_handle.unwrap();

        let mut songs: Vec<Song> = app.get_songs().iter().collect();
        match youtube_downloader.get_videos_infos(&url).await {
            Ok(song_info) => {
                songs.push(Song {
                    title: SharedString::from(song_info.title),
                    downloading: true,
                });
            }
            Err(e) => eprintln!("Erreur lors de la récupération des informations de la vidéo : {}", e),
        }

        app.set_songs(ModelRc::new(VecModel::from(songs)));
    });

    app.run().unwrap();

    Ok(())
}
