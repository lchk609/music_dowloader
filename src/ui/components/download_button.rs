use crate::App;
use crate::Song;
use crate::YoutubeDownloader;
use slint::{ComponentHandle, Model, SharedString, VecModel};
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
                let app_handle_for_invoke = app_handle.clone();
                let youtube_downloader = Arc::clone(&youtube_downloader);

                let video_title = match youtube_downloader
                    .dowload_audio_stream_with_progress(&url, move |video_title: &str| {
                        Box::new(download_callback(
                            app_handle_for_invoke.clone(),
                            video_title.to_string(),
                        ))
                    })
                    .await
                {
                    Ok(video_title) => video_title,
                    Err(e) => {
                        eprintln!("Error downloading audio stream: {}", e);
                        return;
                    }
                };

                slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_handle.upgrade() {
                        let model = app.get_songs();

                        if let Some(vec_model) = model.as_any().downcast_ref::<VecModel<Song>>() {
                            for i in 0..vec_model.row_count() {
                                if let Some(mut song) = vec_model.row_data(i) {
                                    if song.title == video_title {
                                        song.is_downloading = false;
                                        vec_model.set_row_data(i, song);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                })
                .unwrap();
            });
        }
    })
}

pub fn download_callback(
    app_handle: slint::Weak<App>,
    video_title: String,
) -> impl Fn(u64, u64) + Send + Sync + 'static {
    move |_downloaded: u64, _total: u64| {
        let app_weak = app_handle.clone();
        let title = video_title.clone();

        slint::invoke_from_event_loop(move || {
            if let Some(app) = app_weak.upgrade() {
                let model = app.get_songs();

                if let Some(vec_model) = model.as_any().downcast_ref::<VecModel<Song>>() {
                    let exists = (0..vec_model.row_count())
                        .filter_map(|i| vec_model.row_data(i))
                        .any(|s| s.title == title);

                    if !exists {
                        vec_model.insert(
                            0,
                            Song {
                                title: title.clone().into(),
                                is_downloading: true,
                            },
                        );
                    }
                }
            }
        })
        .unwrap();
    }
}
