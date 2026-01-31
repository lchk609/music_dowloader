use std::io;
use url::{Url};

mod youtube;
mod codec;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    youtube::dowload_tools().await?;
    println!("Please enter the music URL:");

    let mut music_url: String = String::new();

    let parsed_url: Url = loop {
        music_url.clear();
        io::stdin()
            .read_line(&mut music_url)
            .expect("Failed to read line");
        match Url::parse(&music_url) {
            Ok(url) => break url,
            Err(e) => {
                eprintln!("Error parsing URL: {}", e);
                continue;
            }
        };
    };

    youtube::download_audio_stream_from_url(parsed_url.to_string(), codec::CODEC_PREFERENCE::MP3, String::new()).await?;

    Ok(())
}
