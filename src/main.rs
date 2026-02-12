use dowloaders::youtube::YoutubeDownloader;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::sync::Arc;

mod dowloaders;
mod enums;

slint::include_modules!();

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let youtube_downloader: Arc<YoutubeDownloader> = Arc::new(YoutubeDownloader::new(
        std::path::PathBuf::new(),
        enums::codec::CODEC_PREFERENCE::MP3,
        3,
    ));
    youtube_downloader.dowload_tools().await?;
    let app = App::new().unwrap();

    manage_add_music(&app, Arc::clone(&youtube_downloader)).await;
    app.run().unwrap();

    Ok(())
}

async fn manage_add_music(app: &App, youtube_downloader: Arc<YoutubeDownloader>) {
    app.on_download_clicked({
        let app_handle = app.as_weak();
        let youtube_downloader = Arc::clone(&youtube_downloader);
        move |url: SharedString| {
            let youtube_downloader = Arc::clone(&youtube_downloader);
            let url = url.to_string();
            let app_handle = app_handle.clone();

            tokio::spawn(async move {
                let youtube_downloader = Arc::clone(&youtube_downloader);
                let video_info = match youtube_downloader.get_videos_infos(&url).await {
                    Ok(info) => info,
                    Err(e) => {
                        eprintln!("Erreur infos : {}", e);
                        return;
                    }
                };

                let app_handle_for_progress = app_handle.clone();
                let video_title_for_progress = video_info.title.clone();

                // let youtube_downloader = Arc::clone(&youtube_downloader);
                let dowload_id = match youtube_downloader
                    .dowload_audio_stream_with_progress(&url,  move |downloaded: u64, total: u64| {
                        let app_handle_for_invoke = app_handle_for_progress.clone();
                        let title_for_invoke = video_title_for_progress.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(app) = app_handle_for_invoke.upgrade() {
                                let mut songs: Vec<Song> = app.get_songs().iter().collect();
                                if let Some(song) =
                                    songs.iter_mut().find(|s| s.title == title_for_invoke)
                                {
                                    song.download_progress = (downloaded * 100 / total) as i32;
                                }
                                app.set_songs(ModelRc::new(VecModel::from(songs)));
                            }
                        })
                        .unwrap();
                    })
                    .await 
                    { 
                        Ok(id) => id, 
                        Err(e) => { 
                            eprintln!("Erreur téléchargement : {}", e); 
                            return; 
                        }
                    };

                let video_title = video_info.title.clone(); // Clone before first move
                let app_handle_clone = app_handle.clone(); // Clone for first closure

                slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_handle_clone.upgrade() {
                        let mut songs: Vec<Song> = app.get_songs().iter().collect();

                        songs.push(Song {
                            title: SharedString::from(video_title.clone()),
                            download_id: dowload_id as i32,
                            total: 100,
                            download_progress: 0,
                        });

                        app.set_songs(ModelRc::new(VecModel::from(songs)));
                    }
                })
                .unwrap();
            });
        }
    })
}
