use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CODEC_PREFERENCE {
    FLAC,
    #[default]
    MP3,
    AAC,
    WAV,
}

impl ToString for CODEC_PREFERENCE {
    fn to_string(&self) -> String {
        match self {
            CODEC_PREFERENCE::FLAC => "flac".to_string(),
            CODEC_PREFERENCE::MP3 => "mp3".to_string(),
            CODEC_PREFERENCE::AAC => "aac".to_string(),
            CODEC_PREFERENCE::WAV => "wav".to_string(),
        }
    }
}
