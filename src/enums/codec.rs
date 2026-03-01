use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CodecPreference {
    FLAC,
    #[default]
    MP3,
    AAC,
    WAV,
}

impl ToString for CodecPreference {
    fn to_string(&self) -> String {
        match self {
            CodecPreference::FLAC => "flac".to_string(),
            CodecPreference::MP3 => "mp3".to_string(),
            CodecPreference::AAC => "aac".to_string(),
            CodecPreference::WAV => "wav".to_string(),
        }
    }
}
