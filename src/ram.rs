use super::convert;

/// A resource of raw PCM samples stored in RAM. This struct stores samples
/// in their native sample format when possible to save memory.
///
/// All channels are de-interleaved.
pub struct PcmRAM {
    pcm_type: PcmRAMType,
    sample_rate: u32,
    channels: usize,
    len_frames: usize,
}

/// The format of the raw PCM samples store in RAM.
///
/// All channels are de-interleaved.
///
/// Note that there is no option for U32/I32. This is because we want to use
/// float for everything anyway. We only store the other types to save memory.
pub enum PcmRAMType {
    U8(Vec<Vec<u8>>),
    U16(Vec<Vec<u16>>),
    /// The endianness of the samples must be the native endianness of the
    /// target platform.
    U24(Vec<Vec<[u8; 3]>>),
    S8(Vec<Vec<i8>>),
    S16(Vec<Vec<i16>>),
    /// The endianness of the samples must be the native endianness of the
    /// target platform.
    S24(Vec<Vec<[u8; 3]>>),
    F32(Vec<Vec<f32>>),
    F64(Vec<Vec<f64>>),
}

impl PcmRAM {
    pub fn new(pcm_type: PcmRAMType, sample_rate: u32) -> Self {
        let (channels, len_frames) = match &pcm_type {
            PcmRAMType::U8(b) => {
                let len = b[0].len();

                for ch in b.iter().skip(1) {
                    assert_eq!(ch.len(), len);
                }

                (b.len(), len)
            }
            PcmRAMType::U16(b) => {
                let len = b[0].len();

                for ch in b.iter().skip(1) {
                    assert_eq!(ch.len(), len);
                }

                (b.len(), len)
            }
            PcmRAMType::U24(b) => {
                let len = b[0].len();

                for ch in b.iter().skip(1) {
                    assert_eq!(ch.len(), len);
                }

                (b.len(), len)
            }
            PcmRAMType::S8(b) => {
                let len = b[0].len();

                for ch in b.iter().skip(1) {
                    assert_eq!(ch.len(), len);
                }

                (b.len(), len)
            }
            PcmRAMType::S16(b) => {
                let len = b[0].len();

                for ch in b.iter().skip(1) {
                    assert_eq!(ch.len(), len);
                }

                (b.len(), len)
            }
            PcmRAMType::S24(b) => {
                let len = b[0].len();

                for ch in b.iter().skip(1) {
                    assert_eq!(ch.len(), len);
                }

                (b.len(), len)
            }
            PcmRAMType::F32(b) => {
                let len = b[0].len();

                for ch in b.iter().skip(1) {
                    assert_eq!(ch.len(), len);
                }

                (b.len(), len)
            }
            PcmRAMType::F64(b) => {
                let len = b[0].len();

                for ch in b.iter().skip(1) {
                    assert_eq!(ch.len(), len);
                }

                (b.len(), len)
            }
        };

        Self {
            pcm_type,
            sample_rate,
            channels,
            len_frames,
        }
    }

    /// The number of channels in this resource.
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// The length of this resource in frames (length of a single channel in
    /// samples).
    pub fn len_frames(&self) -> usize {
        self.len_frames
    }

    /// The sample rate of this resource in samples per second.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get(&self) -> &PcmRAMType {
        &self.pcm_type
    }

    /// Fill the buffer with samples from the given `channel`, starting from the
    /// given `frame`.
    ///
    /// If the length of the buffer exceeds the length of the PCM resource, then
    /// the remaining samples will be filled with zeros.
    ///
    /// This returns the number of frames that were copied into the buffer. (If
    /// this number is less than the length of `buf`, then it means that the
    /// remaining samples were filled with zeros.)
    ///
    /// The will return an error if the given channel does not exist.
    pub fn fill_channel_f32(
        &self,
        channel: usize,
        frame: usize,
        buf: &mut [f32],
    ) -> Result<usize, ()> {
        if channel >= self.channels {
            return Err(());
        }

        if frame >= self.len_frames {
            // Out of range, fill with zeros instead.
            buf.fill(0.0);
            return Ok(0);
        }

        let fill_frames = if frame + buf.len() > self.len_frames {
            // Fill the out-of-range part with zeros.
            let fill_frames = self.len_frames - frame;
            buf[fill_frames..].fill(0.0);
            fill_frames
        } else {
            buf.len()
        };

        let buf_part = &mut buf[0..fill_frames];

        match &self.pcm_type {
            PcmRAMType::U8(pcm) => {
                let pcm_part = &pcm[channel][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_part[i] = convert::pcm_u8_to_f32(pcm_part[i]);
                }
            }
            PcmRAMType::U16(pcm) => {
                let pcm_part = &pcm[channel][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_part[i] = convert::pcm_u16_to_f32(pcm_part[i]);
                }
            }
            PcmRAMType::U24(pcm) => {
                let pcm_part = &pcm[channel][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_part[i] = convert::pcm_u24_to_f32_ne(pcm_part[i]);
                }
            }
            PcmRAMType::S8(pcm) => {
                let pcm_part = &pcm[channel][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_part[i] = convert::pcm_i8_to_f32(pcm_part[i]);
                }
            }
            PcmRAMType::S16(pcm) => {
                let pcm_part = &pcm[channel][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_part[i] = convert::pcm_i16_to_f32(pcm_part[i]);
                }
            }
            PcmRAMType::S24(pcm) => {
                let pcm_part = &pcm[channel][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_part[i] = convert::pcm_i24_to_f32_ne(pcm_part[i]);
                }
            }
            PcmRAMType::F32(pcm) => {
                let pcm_part = &pcm[channel][frame..frame + fill_frames];

                buf_part.copy_from_slice(pcm_part);
            }
            PcmRAMType::F64(pcm) => {
                let pcm_part = &pcm[channel][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_part[i] = pcm_part[i] as f32;
                }
            }
        }

        Ok(fill_frames)
    }

    /// Fill the stereo buffer with samples, starting from the given `frame`.
    ///
    /// If this resource has only one channel, then both channels will be
    /// filled with the same data.
    ///
    /// If the length of the buffer exceeds the length of the PCM resource, then
    /// the remaining samples will be filled with zeros.
    ///
    /// This returns the number of frames that were copied into the buffer. (If
    /// this number is less than the length of `buf`, then it means that the
    /// remaining samples were filled with zeros.)
    pub fn fill_stereo_f32(&self, frame: usize, buf_l: &mut [f32], buf_r: &mut [f32]) -> usize {
        let buf_len = buf_l.len().min(buf_r.len());

        if self.channels == 1 {
            let fill_frames = self.fill_channel_f32(0, frame, buf_l).unwrap();
            buf_r.copy_from_slice(buf_l);
            return fill_frames;
        }

        if frame >= self.len_frames {
            // Out of range, fill with zeros instead.
            buf_l.fill(0.0);
            buf_r.fill(0.0);
            return 0;
        }

        let fill_frames = if frame + buf_len > self.len_frames {
            // Fill the out-of-range part with zeros.
            let fill_frames = self.len_frames - frame;
            buf_l[fill_frames..].fill(0.0);
            buf_r[fill_frames..].fill(0.0);
            fill_frames
        } else {
            buf_len
        };

        let buf_l_part = &mut buf_l[0..fill_frames];
        let buf_r_part = &mut buf_r[0..fill_frames];

        match &self.pcm_type {
            PcmRAMType::U8(pcm) => {
                let pcm_l_part = &pcm[0][frame..frame + fill_frames];
                let pcm_r_part = &pcm[1][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_l_part[i] = convert::pcm_u8_to_f32(pcm_l_part[i]);
                    buf_r_part[i] = convert::pcm_u8_to_f32(pcm_r_part[i]);
                }
            }
            PcmRAMType::U16(pcm) => {
                let pcm_l_part = &pcm[0][frame..frame + fill_frames];
                let pcm_r_part = &pcm[1][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_l_part[i] = convert::pcm_u16_to_f32(pcm_l_part[i]);
                    buf_r_part[i] = convert::pcm_u16_to_f32(pcm_r_part[i]);
                }
            }
            PcmRAMType::U24(pcm) => {
                let pcm_l_part = &pcm[0][frame..frame + fill_frames];
                let pcm_r_part = &pcm[1][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_l_part[i] = convert::pcm_u24_to_f32_ne(pcm_l_part[i]);
                    buf_r_part[i] = convert::pcm_u24_to_f32_ne(pcm_r_part[i]);
                }
            }
            PcmRAMType::S8(pcm) => {
                let pcm_l_part = &pcm[0][frame..frame + fill_frames];
                let pcm_r_part = &pcm[1][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_l_part[i] = convert::pcm_i8_to_f32(pcm_l_part[i]);
                    buf_r_part[i] = convert::pcm_i8_to_f32(pcm_r_part[i]);
                }
            }
            PcmRAMType::S16(pcm) => {
                let pcm_l_part = &pcm[0][frame..frame + fill_frames];
                let pcm_r_part = &pcm[1][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_l_part[i] = convert::pcm_i16_to_f32(pcm_l_part[i]);
                    buf_r_part[i] = convert::pcm_i16_to_f32(pcm_r_part[i]);
                }
            }
            PcmRAMType::S24(pcm) => {
                let pcm_l_part = &pcm[0][frame..frame + fill_frames];
                let pcm_r_part = &pcm[1][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_l_part[i] = convert::pcm_i24_to_f32_ne(pcm_l_part[i]);
                    buf_r_part[i] = convert::pcm_i24_to_f32_ne(pcm_r_part[i]);
                }
            }
            PcmRAMType::F32(pcm) => {
                let pcm_l_part = &pcm[0][frame..frame + fill_frames];
                let pcm_r_part = &pcm[1][frame..frame + fill_frames];

                buf_l_part.copy_from_slice(pcm_l_part);
                buf_r_part.copy_from_slice(pcm_r_part);
            }
            PcmRAMType::F64(pcm) => {
                let pcm_l_part = &pcm[0][frame..frame + fill_frames];
                let pcm_r_part = &pcm[1][frame..frame + fill_frames];

                for i in 0..fill_frames {
                    buf_l_part[i] = pcm_l_part[i] as f32;
                    buf_r_part[i] = pcm_r_part[i] as f32;
                }
            }
        }

        fill_frames
    }

    /// Consume this resource and return the raw samples.
    pub fn to_raw(self) -> PcmRAMType {
        self.pcm_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcm_fill_range_test() {
        let test_pcm = PcmRAM::new(PcmRAMType::F32(vec![vec![1.0, 2.0, 3.0, 4.0]]), 44100);

        let mut out_buf: [f32; 8] = [10.0; 8];

        let fill_frames = test_pcm.fill_channel_f32(0, 0, &mut out_buf[0..4]);
        assert_eq!(fill_frames, Ok(4));
        assert_eq!(&out_buf[0..4], &[1.0, 2.0, 3.0, 4.0]);

        out_buf = [10.0; 8];
        let fill_frames = test_pcm.fill_channel_f32(0, 0, &mut out_buf[0..5]);
        assert_eq!(fill_frames, Ok(4));
        assert_eq!(&out_buf[0..5], &[1.0, 2.0, 3.0, 4.0, 0.0]);

        out_buf = [10.0; 8];
        let fill_frames = test_pcm.fill_channel_f32(0, 2, &mut out_buf[0..4]);
        assert_eq!(fill_frames, Ok(2));
        assert_eq!(&out_buf[0..4], &[3.0, 4.0, 0.0, 0.0]);

        out_buf = [10.0; 8];
        let fill_frames = test_pcm.fill_channel_f32(0, 3, &mut out_buf[0..4]);
        assert_eq!(fill_frames, Ok(1));
        assert_eq!(&out_buf[0..4], &[4.0, 0.0, 0.0, 0.0]);

        out_buf = [10.0; 8];
        let fill_frames = test_pcm.fill_channel_f32(0, 4, &mut out_buf[0..4]);
        assert_eq!(fill_frames, Ok(0));
        assert_eq!(&out_buf[0..4], &[0.0, 0.0, 0.0, 0.0]);
    }
}
