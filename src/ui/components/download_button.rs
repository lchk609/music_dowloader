use crate::App;
use crate::Song;
use crate::YoutubeDownloader;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::sync::Arc;

pub async fn manage_add_music(app: &App, youtube_downloader: Arc<YoutubeDownloader>) {
    app.on_download_clicked({
        let app_handle = app.as_weak();
        let youtube_downloader = Arc::clone(&youtube_downloader);
        move |url: SharedString| {
            let youtube_downloader = Arc::clone(&youtube_downloader);
            let url = url.to_string();
            let app_handle = app_handle.clone();

            tokio::spawn(async move {
                let app_handle = app_handle.clone();
                let youtube_downloader = Arc::clone(&youtube_downloader);
                let video_info = match youtube_downloader.get_videos_infos(&url).await {
                    Ok(info) => info,
                    Err(e) => {
                        eprintln!("Erreur infos : {}", e);
                        return;
                    }
                };

                let app_handle: Arc<slint::Weak<App>> = Arc::new(app_handle.clone());
                let video_title: Arc<String> = Arc::new(video_info.title.clone());

                if let Err(e) = youtube_downloader
                    .dowload_audio_stream_with_progress(&url, move |downloaded: u64, total: u64| {
                        let app_handle_for_invoke = app_handle.clone();
                        let title_for_invoke = video_title.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(app) = app_handle_for_invoke.upgrade() {
                                let mut songs: Vec<Song> = app.get_songs().iter().collect();
                                if None
                                    == songs.iter_mut().find(|s: &&mut Song| {
                                        s.title == SharedString::from(title_for_invoke.as_str())
                                    })
                                {
                                    println!("Adding song to UI: {}", &title_for_invoke);
                                    songs.push(Song {
                                        title: SharedString::from(String::from(
                                            title_for_invoke.as_str(),
                                        )),
                                        download_id: 0,
                                        total: 100,
                                        download_progress: 0,
                                    });
                                    app.set_songs(ModelRc::new(VecModel::from(songs)));
                                }
                            }
                        })
                        .unwrap();

                        let app_handle_for_invoke = app_handle.clone();
                        let title_for_invoke = video_title.clone();

                        slint::invoke_from_event_loop(move || {
                            if let Some(app) = app_handle_for_invoke.upgrade() {
                                let mut songs: Vec<Song> = app.get_songs().iter().collect();
                                if let Some(song) = songs.iter_mut().find(|s: &&mut Song| {
                                    s.title == SharedString::from(title_for_invoke.as_str())
                                }) {
                                    println!("Updating song progress: {}", &song.title);
                                    song.total = total as i32;
                                    song.download_progress = (downloaded * 100 / total) as i32;
                                    println!(
                                        "Download progress for '{}': {} / {}",
                                        title_for_invoke, downloaded, total
                                    );
                                }
                                app.set_songs(ModelRc::new(VecModel::from(songs)));
                            }
                        })
                        .unwrap();
                    })
                    .await
                {
                    eprintln!("Erreur téléchargement : {}", e);
                };
            });
        }
    })
}
