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
    pub fn to_codec(string: &str) -> CodecPreference {
        match string {
            "flac" => CodecPreference::FLAC,
            "mp3" => CodecPreference::MP3,
            "aac" => CodecPreference::AAC,
            "wav" => CodecPreference::WAV,
            "ogg" => CodecPreference::OGG,
            _ => CodecPreference::MP3,
        }
    }
}
