use std::sync::Arc;
use slint::{Model, ModelRc, VecModel, SharedString};
use dowloaders::youtube::{YoutubeDownloader};

mod dowloaders;
mod enums;

slint::include_modules!();

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

    app.on_download_clicked({
        let youtube_downloader = Arc::clone(&youtube_downloader);
        let app_handle = app_handle.clone();
        move |url: SharedString| {
            let url = url.to_string();
            let youtube_downloader = Arc::clone(&youtube_downloader);
            let app_handle = app_handle.clone();

            tokio::spawn(async move {
                let youtube_downloader = Arc::clone(&youtube_downloader);
                let video_info = match youtube_downloader
                    .get_videos_infos(&url).await {
                        Ok(info) => info,
                        Err(e) => {
                            eprintln!("Erreur infos : {}", e);
                            return;
                        }
                    };

                slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_handle.upgrade() {
                        let mut songs: Vec<Song> = app.get_songs().iter().collect();

                        songs.push(Song {
                            title: SharedString::from(video_info.title),
                            downloading: true,
                        });

                        app.set_songs(ModelRc::new(VecModel::from(songs)));
                    }
                }).unwrap();

                let youtube_downloader = Arc::clone(&youtube_downloader);
                if let Err(e) = youtube_downloader
                    .download_audio_stream_from_url(&url)
                    .await
                {
                    eprintln!("Erreur download : {}", e);
                }
            });
        }
    });

    app.run().unwrap();

    Ok(())
}
