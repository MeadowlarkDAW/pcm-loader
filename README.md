# pcm-loader (name in progress)

Easily load audio files into RAM (WIP)

This is mostly an easy-to-use wrapper around the [`Symphonia`] decoding library. This crate also handles resampling to a target sample rate either at load-time or in realtime during playback.

The resulting `PcmRAM` resources are always de-interleaved, and they are stored in their native sample format when possible to save memory. They also have convenience methods to fill de-interleaved `f32` output buffers from any aribtrary position in the resource.

(TODO: Example)