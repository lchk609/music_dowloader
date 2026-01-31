#[derive(PartialEq, Eq)]
pub enum CODEC_PREFERENCE {
    FLAC,
    MP3,
    AAC,
    WAV,
}

pub fn get_codec_extension(codec: &CODEC_PREFERENCE) -> &'static str {
    match codec {
        CODEC_PREFERENCE::FLAC => "flac",
        CODEC_PREFERENCE::MP3 => "mp3",
        CODEC_PREFERENCE::AAC => "aac",
        CODEC_PREFERENCE::WAV => "wav",
    }
}
