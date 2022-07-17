use std::fs::File;
use std::path::Path;

use fnv::FnvHashMap;
use samplerate_rs::ConverterType;
use symphonia::core::codecs::CodecRegistry;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::{Hint, Probe};

pub mod convert;
pub mod error;

mod decode;
mod ram;

pub use ram::*;

use error::PcmLoadError;

pub static DEFAULT_MAX_BYTES: usize = 1_000_000_000;

/// A converter type used to distinguish the interpolation function used by libsamplerate.
/// Has a great impact on quality and performance.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum ResampleQuality {
    /// Best quality, slowest
    SincBestQuality,
    /// 2nd best quality, 2nd slowest
    SincMediumQuality,
    /// 3rd best quality, 3rd slowest
    SincFastest,
    /// 4th best quality, 2nd fastest (signinficantly faster than Sinc)
    Linear,
    /// Worst quality, fastest
    ZeroOrderHold,
}

impl ResampleQuality {
    /// Return a human-readable name for this type of converter.
    pub fn name(&self) -> &'static str {
        let c: ConverterType = (*self).into();
        c.name()
    }

    /// Return the human-readable description for this type of converter.
    pub fn description(&self) -> &'static str {
        let c: ConverterType = (*self).into();
        c.description()
    }
}

impl Default for ResampleQuality {
    fn default() -> Self {
        ResampleQuality::SincFastest
    }
}

impl From<ConverterType> for ResampleQuality {
    fn from(c: ConverterType) -> Self {
        match c {
            ConverterType::SincBestQuality => ResampleQuality::SincBestQuality,
            ConverterType::SincMediumQuality => ResampleQuality::SincMediumQuality,
            ConverterType::SincFastest => ResampleQuality::SincFastest,
            ConverterType::ZeroOrderHold => ResampleQuality::ZeroOrderHold,
            ConverterType::Linear => ResampleQuality::Linear,
        }
    }
}

impl From<ResampleQuality> for ConverterType {
    fn from(r: ResampleQuality) -> Self {
        match r {
            ResampleQuality::SincBestQuality => ConverterType::SincBestQuality,
            ResampleQuality::SincMediumQuality => ConverterType::SincMediumQuality,
            ResampleQuality::SincFastest => ConverterType::SincFastest,
            ResampleQuality::ZeroOrderHold => ConverterType::ZeroOrderHold,
            ResampleQuality::Linear => ConverterType::Linear,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ResamplerKey {
    pcm_sr: u32,
    target_sr: u32,
    channels: u32,
    quality: ResampleQuality,
}

/// Used to load audio files into RAM. This stores samples in
/// their native sample format when possible to save memory.
pub struct PcmLoader {
    // Re-use resamplers to improve performance.
    resamplers: FnvHashMap<ResamplerKey, samplerate_rs::Samplerate>,

    codec_registry: &'static CodecRegistry,
    probe: &'static Probe,
}

impl PcmLoader {
    /// Construct a new audio file loader.
    pub fn new() -> Self {
        Self {
            resamplers: FnvHashMap::default(),
            codec_registry: symphonia::default::get_codecs(),
            probe: symphonia::default::get_probe(),
        }
    }

    /// Load the audio file from the given path into RAM.
    ///
    /// * `audio_file_path` - The path to the audio file stored on disk.
    /// * `target_sample_rate` - If this is `Some`, then the file will be resampled to that
    /// sample rate. (No resampling will occur if the audio file's sample rate is already
    /// the target sample rate). If this is `None`, then the file will not be resampled
    /// and it will stay its original sample rate.
    ///     * Note that resampling will always convert the sample format to `f32`. If
    /// saving memory is a concern, then set this to `None` and resample in realtime.
    /// (Realtime resampling is not implemented in this crate yet).
    /// * `resample_quality` - The quality of the resampling. This will have an impact on
    /// both decoding speed and audio quality. This will be ignored if the audio file's
    /// sample rate is already the target sample rate, or if `target_sample_rate` is `None`.
    /// * `max_bytes` - The maximum size in bytes that the resulting `PcmRAM` resource can
    /// be in RAM. If the resulting resource is larger than this, then an error will be
    /// returned instead. This is useful to avoid locking up or crashing the system if the
    /// use tries to load a really large audio file.
    ///     * If this is `None`, then default of `1_000_000_000` (1GB) will be used.
    pub fn load<P: AsRef<Path>>(
        &mut self,
        audio_file_path: P,
        target_sample_rate: Option<u32>,
        resample_quality: ResampleQuality,
        max_bytes: Option<usize>,
    ) -> Result<PcmRAM, PcmLoadError> {
        let audio_file_path: &Path = audio_file_path.as_ref();

        // Try to open the file.
        let file = File::open(audio_file_path)?;

        // Create a hint to help the format registry guess what format reader is appropriate.
        let mut hint = Hint::new();

        // Provide the file extension as a hint.
        if let Some(extension) = audio_file_path.extension() {
            if let Some(extension_str) = extension.to_str() {
                hint.with_extension(extension_str);
            }
        }

        // Create the media source stream.
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Use the default options for format reader, metadata reader, and decoder.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();

        // Probe the media source stream for metadata and get the format reader.
        let mut probed = self
            .probe
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| PcmLoadError::UnkownFormat(e))?;

        // Get the default track in the audio stream.
        let track = probed
            .format
            .default_track()
            .ok_or_else(|| PcmLoadError::NoTrackFound)?;

        let sample_rate = track.codec_params.sample_rate.unwrap_or_else(|| {
            log::warn!(
                "Could not find sample rate of PCM resource. Assuming a sample rate of 44100"
            );
            44100
        });

        let n_channels = track
            .codec_params
            .channels
            .ok_or_else(|| PcmLoadError::NoChannelsFound)?
            .count();

        if let Some(target_sr) = target_sample_rate {
            if sample_rate != target_sr {
                // Resampling is needed.

                let resampler_key = ResamplerKey {
                    pcm_sr: sample_rate,
                    target_sr,
                    channels: n_channels as u32,
                    quality: resample_quality,
                };

                let mut resampler = self.resamplers.get_mut(&resampler_key);

                if resampler.is_none() {
                    let new_rs = samplerate_rs::Samplerate::new(
                        resample_quality.into(),
                        sample_rate,
                        target_sr,
                        n_channels,
                    )?;

                    let _ = self.resamplers.insert(resampler_key, new_rs);

                    resampler = self.resamplers.get_mut(&resampler_key);
                }

                let resampler = resampler.as_mut().unwrap();

                resampler.reset().unwrap();

                let pcm = decode::decode_f32_resampled(
                    &mut probed,
                    self.codec_registry,
                    sample_rate,
                    target_sr,
                    n_channels,
                    resampler,
                    max_bytes.unwrap_or(DEFAULT_MAX_BYTES),
                )?;

                return Ok(pcm);
            }
        }

        let pcm = decode::decode_native_bitdepth(
            &mut probed,
            n_channels,
            self.codec_registry,
            sample_rate,
            max_bytes.unwrap_or(DEFAULT_MAX_BYTES),
        )?;

        Ok(pcm)
    }
}
