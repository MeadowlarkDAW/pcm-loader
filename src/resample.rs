use std::{collections::HashMap, fmt::Debug};

// Re-export rubato
pub use rubato;

use rubato::{
    FastFixedIn, PolynomialDegree, ResampleResult, Resampler, SincFixedIn,
    SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

#[cfg(feature = "fft-resampler")]
use rubato::FftFixedIn;

/// The quality of the resampling algorithm to use.
#[derive(Default)]
pub enum ResampleQuality<'a> {
    /// Low quality, fast performance
    ///
    /// More specifically, this uses the [`FastFixedIn`] resampler from
    /// rubato with an interpolation type of [`PolynomialDegree::Linear`]
    /// and a chunk size of `1024`.
    Low,
    /// Good quality, medium performance
    ///
    /// This is recommended for most applications.
    ///
    /// More specifically, if the `fft` feature is enabled (which it is by default),
    /// then this uses the [`FftFixedIn`] resampler from rubato with a chunk size of
    /// `1024` and 2 sub chunks.
    ///
    /// If the `fft` feature is not enabled then this uses the [`FastFixedIn`]
    /// resampler from rubato with an interpolation type of
    /// [`PolynomialDegree::Quintic`] and a chunk size of `1024`.
    #[default]
    Normal,
    /// High quality, slow performance
    ///
    /// More specifically, this uses the [`SincFixedIn`] resampler from
    /// rubato with the following parameters:
    ///
    /// ```ignore
    /// SincInterpolationParameters {
    ///     sinc_len: 128,
    ///     f_cutoff: rubato::calculate_cutoff(128, WindowFunction::Blackman2),
    ///     interpolation: SincInterpolationType::Cubic,
    ///     oversampling_factor: 256,
    ///     window: WindowFunction::Blackman2,
    /// }
    /// ```
    High,
    /// Use a custom resampler
    Custom(ResamplerRefMut<'a>),
}

impl<'a> Debug for ResampleQuality<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResampleQuality::Low => write!(f, "Low"),
            ResampleQuality::Normal => write!(f, "Normal"),
            ResampleQuality::High => write!(f, "High"),
            ResampleQuality::Custom(_) => write!(f, "Custom"),
        }
    }
}

/// A reference to a custom resampler.
pub enum ResamplerRefMut<'a> {
    Fast(&'a mut FastFixedIn<f32>),
    #[cfg(feature = "fft-resampler")]
    Fft(&'a mut FftFixedIn<f32>),
    Sinc(&'a mut SincFixedIn<f32>),
}

impl<'a> ResamplerRefMut<'a> {
    pub fn num_channels(&self) -> usize {
        match self {
            Self::Fast(r) => r.nbr_channels(),
            #[cfg(feature = "fft-resampler")]
            Self::Fft(r) => r.nbr_channels(),
            Self::Sinc(r) => r.nbr_channels(),
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Fast(r) => r.reset(),
            #[cfg(feature = "fft-resampler")]
            Self::Fft(r) => r.reset(),
            Self::Sinc(r) => r.reset(),
        }
    }

    pub fn input_frames_next(&mut self) -> usize {
        match self {
            Self::Fast(r) => r.input_frames_next(),
            #[cfg(feature = "fft-resampler")]
            Self::Fft(r) => r.input_frames_next(),
            Self::Sinc(r) => r.input_frames_next(),
        }
    }

    pub fn input_frames_max(&mut self) -> usize {
        match self {
            Self::Fast(r) => r.input_frames_max(),
            #[cfg(feature = "fft-resampler")]
            Self::Fft(r) => r.input_frames_max(),
            Self::Sinc(r) => r.input_frames_max(),
        }
    }

    pub fn output_delay(&mut self) -> usize {
        match self {
            Self::Fast(r) => r.output_delay(),
            #[cfg(feature = "fft-resampler")]
            Self::Fft(r) => r.output_delay(),
            Self::Sinc(r) => r.output_delay(),
        }
    }

    pub fn output_frames_max(&mut self) -> usize {
        match self {
            Self::Fast(r) => r.output_frames_max(),
            #[cfg(feature = "fft-resampler")]
            Self::Fft(r) => r.output_frames_max(),
            Self::Sinc(r) => r.output_frames_max(),
        }
    }

    pub fn process_into_buffer<Vin: AsRef<[f32]>, Vout: AsMut<[f32]>>(
        &mut self,
        wave_in: &[Vin],
        wave_out: &mut [Vout],
        active_channels_mask: Option<&[bool]>,
    ) -> ResampleResult<(usize, usize)> {
        match self {
            Self::Fast(r) => r.process_into_buffer(wave_in, wave_out, active_channels_mask),
            #[cfg(feature = "fft-resampler")]
            Self::Fft(r) => r.process_into_buffer(wave_in, wave_out, active_channels_mask),
            Self::Sinc(r) => r.process_into_buffer(wave_in, wave_out, active_channels_mask),
        }
    }

    pub fn process_partial_into_buffer<Vin: AsRef<[f32]>, Vout: AsMut<[f32]>>(
        &mut self,
        wave_in: Option<&[Vin]>,
        wave_out: &mut [Vout],
        active_channels_mask: Option<&[bool]>,
    ) -> ResampleResult<(usize, usize)> {
        match self {
            Self::Fast(r) => r.process_partial_into_buffer(wave_in, wave_out, active_channels_mask),
            #[cfg(feature = "fft-resampler")]
            Self::Fft(r) => r.process_partial_into_buffer(wave_in, wave_out, active_channels_mask),
            Self::Sinc(r) => r.process_partial_into_buffer(wave_in, wave_out, active_channels_mask),
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ResampleQualityKey {
    Low,
    Normal,
    High,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ResamplerKey {
    pcm_sr: u32,
    target_sr: u32,
    channels: u32,
    quality: ResampleQualityKey,
}

pub(crate) enum ResamplerOwned {
    Fast(FastFixedIn<f32>),
    #[cfg(feature = "fft-resampler")]
    Fft(FftFixedIn<f32>),
    Sinc(SincFixedIn<f32>),
}

impl ResamplerOwned {
    pub fn as_ref_mut<'a>(&'a mut self) -> ResamplerRefMut<'a> {
        match self {
            Self::Fast(r) => ResamplerRefMut::Fast(r),
            #[cfg(feature = "fft-resampler")]
            Self::Fft(r) => ResamplerRefMut::Fft(r),
            Self::Sinc(r) => ResamplerRefMut::Sinc(r),
        }
    }
}

pub(crate) fn get_resampler<'a>(
    resamplers: &'a mut HashMap<ResamplerKey, ResamplerOwned>,
    resample_quality: ResampleQuality<'a>,
    pcm_sr: u32,
    target_sr: u32,
    n_channels: usize,
) -> ResamplerRefMut<'a> {
    const CHUNK_SIZE: usize = 1024;

    match resample_quality {
        ResampleQuality::Low => resamplers
            .entry(ResamplerKey {
                pcm_sr,
                target_sr,
                channels: n_channels as u32,
                quality: ResampleQualityKey::Low,
            })
            .or_insert_with(|| {
                ResamplerOwned::Fast(
                    FastFixedIn::new(
                        target_sr as f64 / pcm_sr as f64,
                        1.0,
                        PolynomialDegree::Linear,
                        CHUNK_SIZE,
                        n_channels,
                    )
                    .unwrap(),
                )
            })
            .as_ref_mut(),
        ResampleQuality::Normal => resamplers
            .entry(ResamplerKey {
                pcm_sr,
                target_sr,
                channels: n_channels as u32,
                quality: ResampleQualityKey::Normal,
            })
            .or_insert_with(|| {
                #[cfg(feature = "fft-resampler")]
                return ResamplerOwned::Fft(
                    FftFixedIn::new(
                        pcm_sr as usize,
                        target_sr as usize,
                        CHUNK_SIZE,
                        2,
                        n_channels,
                    )
                    .unwrap(),
                );

                #[cfg(not(feature = "fft-resampler"))]
                return ResamplerOwned::Fast(
                    FastFixedIn::new(
                        target_sr as f64 / pcm_sr as f64,
                        1.0,
                        PolynomialDegree::Quintic,
                        CHUNK_SIZE,
                        n_channels,
                    )
                    .unwrap(),
                );
            })
            .as_ref_mut(),
        ResampleQuality::High => resamplers
            .entry(ResamplerKey {
                pcm_sr,
                target_sr,
                channels: n_channels as u32,
                quality: ResampleQualityKey::High,
            })
            .or_insert_with(|| {
                let sinc_len = 128;
                let oversampling_factor = 256;
                let interpolation = SincInterpolationType::Cubic;
                let window = WindowFunction::Blackman2;

                let f_cutoff = rubato::calculate_cutoff(sinc_len, window);
                let params = SincInterpolationParameters {
                    sinc_len,
                    f_cutoff,
                    interpolation,
                    oversampling_factor,
                    window,
                };

                ResamplerOwned::Sinc(
                    SincFixedIn::new(
                        target_sr as f64 / pcm_sr as f64,
                        1.0,
                        params,
                        CHUNK_SIZE,
                        n_channels,
                    )
                    .unwrap(),
                )
            })
            .as_ref_mut(),
        ResampleQuality::Custom(resampler) => {
            if resampler.num_channels() != n_channels {
                // return error
                todo!()
            }

            resampler
        }
    }
}
