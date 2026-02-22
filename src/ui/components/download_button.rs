use crate::App;
use crate::Song;
use crate::YoutubeDownloader;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::collections::VecDeque;
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

                if let Err(e) = youtube_downloader
                    .dowload_audio_stream_with_progress(&url, move |video_title: &str| {
                        Box::new(download_callback(app_handle.clone(), video_title.to_string()))
                    })
                    .await
                {
                    eprintln!("Erreur téléchargement : {}", e);
                };
            });
        }
    })
}

pub fn download_callback(app_handle: slint::Weak<App>, video_title: String) -> impl Fn(u64, u64) + Send + 'static + std::marker::Sync {
    move |downloaded: u64, total: u64| {
    let app_handle_for_invoke = app_handle.clone();
    let title_for_invoke = video_title.clone();
    slint::invoke_from_event_loop(move || {
        if let Some(app) = app_handle_for_invoke.upgrade() {
            let mut songs: VecDeque<Song> = app.get_songs().iter().collect();
            if None
                == songs.iter_mut().find(|s: &&mut Song| {
                    s.title == SharedString::from(title_for_invoke.as_str())
                })
            {
                println!("Adding song to UI: {}", &title_for_invoke);
                songs.push_front(Song {
                    title: SharedString::from(String::from(
                        title_for_invoke.as_str(),
                    )),
                });
                app.set_songs(ModelRc::new(VecModel::from(Vec::from(songs))));
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
                println!(
                    "Download progress for '{}': {} / {}",
                    title_for_invoke, downloaded, total
                );
            }
            app.set_songs(ModelRc::new(VecModel::from(songs)));
        }
    })
    .unwrap();
    }
}
