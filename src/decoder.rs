use anyhow::Result;
use std::io::Cursor;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{CODEC_TYPE_AAC, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Decode AAC audio data to PCM samples
pub fn decode_aac_to_pcm(aac_data: &[u8]) -> Result<Vec<i16>> {
    let owned_data = aac_data.to_vec();
    let cursor = Cursor::new(owned_data);
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("aac");

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();

    let probed =
        match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
            Ok(p) => p,
            Err(e) => {
                log::debug!("Failed to probe AAC data: {}", e);
                return Ok(Vec::new()); // Return empty on probe failure
            }
        };

    let mut format = probed.format;

    let track = match format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec == CODEC_TYPE_AAC)
    {
        Some(t) => t,
        None => {
            log::debug!("No AAC track found");
            return Ok(Vec::new());
        }
    };

    let track_id = track.id;

    let dec_opts = DecoderOptions::default();
    let mut decoder = match symphonia::default::get_codecs().make(&track.codec_params, &dec_opts) {
        Ok(d) => d,
        Err(e) => {
            log::debug!("Failed to create decoder: {}", e);
            return Ok(Vec::new());
        }
    };

    let mut pcm_samples = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                log::debug!("Error reading packet: {}", e);
                break;
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(e) => {
                log::debug!("Decode error: {}", e);
                continue;
            }
        };

        convert_audio_buffer_to_pcm(&decoded, &mut pcm_samples);
    }

    Ok(pcm_samples)
}

fn convert_audio_buffer_to_pcm(audio_buf: &AudioBufferRef, pcm_samples: &mut Vec<i16>) {
    match audio_buf {
        AudioBufferRef::F32(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    let sample = buf.chan(ch)[frame_idx];
                    let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                    pcm_samples.push(sample_i16);
                }
            }
        }
        AudioBufferRef::F64(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    let sample = buf.chan(ch)[frame_idx];
                    let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                    pcm_samples.push(sample_i16);
                }
            }
        }
        AudioBufferRef::S16(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    pcm_samples.push(buf.chan(ch)[frame_idx]);
                }
            }
        }
        AudioBufferRef::S32(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    let sample = buf.chan(ch)[frame_idx];
                    let sample_i16 = (sample >> 16) as i16;
                    pcm_samples.push(sample_i16);
                }
            }
        }
        AudioBufferRef::U8(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    let sample = buf.chan(ch)[frame_idx];
                    let sample_i16 = ((sample as i32 - 128) * 256) as i16;
                    pcm_samples.push(sample_i16);
                }
            }
        }
        AudioBufferRef::U16(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    let sample = buf.chan(ch)[frame_idx];
                    let sample_i16 = (sample as i32 - 32768) as i16;
                    pcm_samples.push(sample_i16);
                }
            }
        }
        AudioBufferRef::U32(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    let sample = buf.chan(ch)[frame_idx];
                    let sample_i16 = ((sample >> 16) as i32 - 32768) as i16;
                    pcm_samples.push(sample_i16);
                }
            }
        }
        AudioBufferRef::S8(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    let sample = buf.chan(ch)[frame_idx];
                    let sample_i16 = (sample as i16) * 256;
                    pcm_samples.push(sample_i16);
                }
            }
        }
        AudioBufferRef::S24(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    let sample = buf.chan(ch)[frame_idx];
                    let sample_i32: i32 = sample.inner();
                    let sample_i16 = (sample_i32 >> 8) as i16;
                    pcm_samples.push(sample_i16);
                }
            }
        }
        AudioBufferRef::U24(buf) => {
            let channels = buf.spec().channels.count();
            let frames = buf.frames();
            for frame_idx in 0..frames {
                for ch in 0..channels {
                    let sample = buf.chan(ch)[frame_idx];
                    let sample_u32: u32 = sample.inner();
                    let sample_i16 = ((sample_u32 >> 8) as i32 - 32768) as i16;
                    pcm_samples.push(sample_i16);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty() {
        let result = decode_aac_to_pcm(&[]);
        assert!(result.is_ok());
    }
}
