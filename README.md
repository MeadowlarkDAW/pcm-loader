# Symphonium

[![Documentation](https://docs.rs/symphonium/badge.svg)](https://docs.rs/symphonium)
[![Crates.io](https://img.shields.io/crates/v/symphonium.svg)](https://crates.io/crates/symphonium)
[![License](https://img.shields.io/crates/l/symphonium.svg)](https://github.com/MeadowlarkDAW/symphonium/blob/main/LICENSE)

An unofficial easy-to-use wrapper around [Symphonia](https://github.com/pdeljanov/Symphonia) for loading audio files. It also handles resampling at load-time.

The resulting `DecodedAudio` resources are stored in their native sample format whenever possible to save on memory, and have convenience methods to fill a buffer with `f32` samples from any arbitrary position in the resource in realtime during playback. Alternatively you can use the `DecodedAudioF32` resource if you only need samples in the `f32` format.

## Example

```rust
/// A struct used to load audio files.
let mut loader = SymphoniumLoader::new();

/// Load an audio file.
let audio_data = loader
    .load(
        // The path to the audio file.
        file_path,    
        // The target sample rate. If this differs from the
        // file's sample rate, then it will be resampled.
        // If you wish to never resample, set this to `None`.
        Some(sample_rate),
        // The quality of the resampling algorithm. Normal
        // is recommended for most applications.
        ResampleQuality::Normal,
        // The maximum size a file can be in bytes before an
        // error is returned. This is to protect against
        // out of memory errors when loading really long
        // audio files. Set to `None` to use the default of
        // 1 GB.
        None
    )
    .unwrap();

/// Fill a stereo buffer with samples starting at frame 100.
let mut buf_l = vec![0.0f32; 512];
let mut buf_r = vec![0.0f32; 512];
audio_data.fill_stereo(100, &mut buf_l, &mut buf_r);

/// Alternatively, if you don't need to save memory, you can
/// load directly to an `f32` format.
let audio_data_f32 = loader
    .load_f32(
        file_path, 
        Some(sample_rate),
        ResampleQuality::Normal,
        None
    )
    .unwrap();

/// Print info about the data (`data` is a `Vec<Vec<f32>>`).
println!("num channels: {}" audio_data_f32.data.len());
println!("num frames: {}" audio_data_f32.data[0].len());
```
## Features

By default, only `wav` and `ogg` support is enabled. If you need more formats, enable them as features in your `Cargo.toml` file like this:

`symphonium = { version = "0.1", features = ["mp3", "flac"] }`

Available codecs:

* `aac`
* `adpcm`
* `alac`
* `flac`
* `mp1`
* `mp2`
* `mp3`
* `pcm`
* `vorbis`

Available container formats:

* `caf`
* `isomp4`
* `mkv`
* `ogg`
* `aiff`
* `wav`

Alternatively you can enable the `all` feature if you want everything, or the `open-standards` feature if you want all of the royalty-free open-source standards.