use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PcmLoadError {
    FileNotFound(std::io::Error),
    UnkownFormat(symphonia::core::errors::Error),
    NoTrackFound,
    NoChannelsFound,
    UnkownChannelFormat(usize),
    FileTooLarge(usize),
    CouldNotCreateDecoder(symphonia::core::errors::Error),
    ErrorWhileDecoding(symphonia::core::errors::Error),
    UnexpectedErrorWhileDecoding(Box<dyn Error>),
    ErrorWhileResampling(samplerate_rs::Error),
}

impl Error for PcmLoadError {}

impl fmt::Display for PcmLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PcmLoadError::*;

        match self {
            FileNotFound(e) => write!(f, "Failed to load PCM resource: file not found | {}", e),
            UnkownFormat(e) => write!(
                f,
                "Failed to load PCM resource: format not supported | {}",
                e
            ),
            NoTrackFound => write!(f, "Failed to load PCM resource: no default track found"),
            NoChannelsFound => write!(f, "Failed to load PCM resource: no channels found"),
            UnkownChannelFormat(n_channels) => write!(
                f,
                "Failed to load PCM resource: unkown channel format | {} channels found",
                n_channels
            ),
            FileTooLarge(max_bytes) => write!(
                f,
                "Failed to load PCM resource: file is too large | maximum is {} bytes",
                max_bytes
            ),
            CouldNotCreateDecoder(e) => write!(
                f,
                "Failed to load PCM resource: failed to create decoder | {}",
                e
            ),
            ErrorWhileDecoding(e) => write!(
                f,
                "Failed to load PCM resource: error while decoding | {}",
                e
            ),
            UnexpectedErrorWhileDecoding(e) => write!(
                f,
                "Failed to load PCM resource: unexpected error while decoding | {}",
                e
            ),
            ErrorWhileResampling(e) => write!(
                f,
                "Failed to load PCM resource: error while resampling | {}",
                e
            ),
        }
    }
}

impl From<std::io::Error> for PcmLoadError {
    fn from(e: std::io::Error) -> Self {
        PcmLoadError::FileNotFound(e)
    }
}

impl From<samplerate_rs::Error> for PcmLoadError {
    fn from(e: samplerate_rs::Error) -> Self {
        PcmLoadError::ErrorWhileResampling(e)
    }
}
