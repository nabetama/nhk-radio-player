use anyhow::Result;
use log::{error, info};
use rodio::buffer::SamplesBuffer;
use std::sync::Arc;

use crate::client::NhkRadioClient;
use crate::crypto::decrypt_segment;
use crate::decoder::decode_aac_to_pcm;
use crate::m3u8::parse_m3u8;

pub struct Player {
    client: Arc<NhkRadioClient>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            client: Arc::new(NhkRadioClient::new()),
        }
    }

    /// Resolve master playlist to actual media playlist URL
    async fn resolve_master_playlist(&self, m3u8_url: &str) -> Result<String> {
        let playlist_content = self.client.fetch_m3u8(m3u8_url).await?;

        match parse_m3u8(&playlist_content, m3u8_url) {
            Ok(_) => Ok(m3u8_url.to_string()),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("Master playlist detected") {
                    if let Some(variant_url) = error_msg.split("Variant URL: ").nth(1) {
                        info!("Detected master playlist, using variant: {}", variant_url);
                        Ok(variant_url.to_string())
                    } else {
                        Err(e)
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Continuously stream and play (for live radio)
    pub async fn play_live(&self, m3u8_url: &str) -> Result<()> {
        info!("Starting live playback from: {}", m3u8_url);

        let actual_url = self.resolve_master_playlist(m3u8_url).await?;
        info!("Resolved stream URL: {}", actual_url);

        let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
        let sink = Arc::new(rodio::Sink::try_new(&stream_handle)?);

        let mut seen_segments = std::collections::HashSet::new();
        let client = self.client.clone();

        loop {
            let playlist_content = match client.fetch_m3u8(&actual_url).await {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to fetch playlist: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            let segments = match parse_m3u8(&playlist_content, &actual_url) {
                Ok(segs) => segs,
                Err(e) => {
                    error!("Failed to parse playlist: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            let key = if let Some(ref seg) = segments.first() {
                if let Some(ref key_url) = seg.key_url {
                    Some(client.fetch_key(key_url).await?)
                } else {
                    None
                }
            } else {
                None
            };

            for segment in segments {
                if seen_segments.contains(&segment.url) {
                    continue;
                }
                seen_segments.insert(segment.url.clone());

                info!("Playing segment: {}", segment.url);

                let mut data = match client.fetch_segment(&segment.url).await {
                    Ok(d) => d,
                    Err(e) => {
                        error!("Failed to fetch segment: {}", e);
                        continue;
                    }
                };

                if let Some(ref k) = key {
                    data = match decrypt_segment(&data, k, segment.iv.as_deref(), segment.seq_no) {
                        Ok(d) => d,
                        Err(e) => {
                            error!("Failed to decrypt: {}", e);
                            continue;
                        }
                    };
                }

                match decode_aac_to_pcm(&data) {
                    Ok(pcm_samples) if !pcm_samples.is_empty() => {
                        let buffer = SamplesBuffer::new(2, 48000, pcm_samples);
                        sink.append(buffer);
                    }
                    Ok(_) => {
                        log::debug!("Decoded empty PCM data");
                    }
                    Err(e) => {
                        log::debug!("Failed to decode AAC: {}", e);
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
