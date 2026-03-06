use serde::{Deserialize, Serialize};

#[derive(Debug,PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub enum CodecPreference {
    FLAC,
    #[default]
    MP3,
    AAC,
    WAV,
    OGG,
}

impl ToString for CodecPreference {
    fn to_string(&self) -> String {
        match self {
            CodecPreference::FLAC => "flac".to_string(),
            CodecPreference::MP3 => "mp3".to_string(),
            CodecPreference::AAC => "aac".to_string(),
            CodecPreference::WAV => "wav".to_string(),
            CodecPreference::OGG => "ogg".to_string(),
        }
    }
}

impl CodecPreference {
    pub fn to_yt_dlp_codec(&self) -> yt_dlp::model::AudioCodecPreference {
        match self {
            CodecPreference::FLAC => yt_dlp::model::AudioCodecPreference::Custom("flac".into()),
            CodecPreference::MP3 => yt_dlp::model::AudioCodecPreference::MP3,
            CodecPreference::AAC => yt_dlp::model::AudioCodecPreference::AAC,
            CodecPreference::WAV => yt_dlp::model::AudioCodecPreference::Custom("wav".into()),
            CodecPreference::OGG => yt_dlp::model::AudioCodecPreference::Custom("ogg".into()),
        }
    }


    pub fn to_yt_dlp_ext(&self) -> String {
        match self {
            CodecPreference::MP3 => "mp3".to_string(),  // yt-dlp utilise souvent m4a pour mp3
            CodecPreference::WAV => "wav".to_string(),
            CodecPreference::FLAC => "flac".to_string(),
            CodecPreference::OGG => "ogg".to_string(),
            CodecPreference::AAC => "m4a".to_string(),  // yt-dlp utilise souvent m4a pour aac
        }
    }

    pub fn to_ffmpeg_codec(&self) -> (&'static str, &'static str) {
        match self {
            CodecPreference::MP3 => ("mp3", "libmp3lame"),
            CodecPreference::OGG => ("ogg", "libvorbis"),
            CodecPreference::FLAC => ("flac", "flac"),
            CodecPreference::WAV => ("wav", "pcm_s16le"),
            CodecPreference::AAC => ("aac", "aac"),
        }
    }
}
