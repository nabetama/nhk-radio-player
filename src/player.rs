use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::watch;

use crate::client::NhkRadioClient;
use crate::crypto::decrypt_segment;
use crate::decoder::decode_aac_to_pcm;
use crate::m3u8::parse_m3u8;
use crate::types::StreamData;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ChannelKind {
    R1,
    R2,
    Fm,
}

impl ChannelKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            ChannelKind::R1 => "ラジオ第1",
            ChannelKind::R2 => "ラジオ第2",
            ChannelKind::Fm => "FM",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            ChannelKind::R1 => "R1",
            ChannelKind::R2 => "R2",
            ChannelKind::Fm => "FM",
        }
    }

    pub fn get_url(&self, data: &StreamData) -> String {
        match self {
            ChannelKind::R1 => data.r1hls.clone(),
            ChannelKind::R2 => data.r2hls.clone(),
            ChannelKind::Fm => data.fmhls.clone(),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ChannelKind::R1 => ChannelKind::R2,
            ChannelKind::R2 => ChannelKind::Fm,
            ChannelKind::Fm => ChannelKind::R1,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            ChannelKind::R1 => ChannelKind::Fm,
            ChannelKind::R2 => ChannelKind::R1,
            ChannelKind::Fm => ChannelKind::R2,
        }
    }
}

/// Handles audio playback in a separate thread
pub fn run_audio_thread(
    rx: std::sync::mpsc::Receiver<Vec<i16>>,
    _channel_rx: watch::Receiver<ChannelKind>,
    playback_notify: std::sync::mpsc::Sender<()>,
) -> Result<()> {
    use rodio::buffer::SamplesBuffer;

    log::info!("Audio thread starting...");

    // Try to get audio output, retry if it fails
    let (stream, sink) = loop {
        match rodio::OutputStream::try_default() {
            Ok((_stream, stream_handle)) => match rodio::Sink::try_new(&stream_handle) {
                Ok(sink) => {
                    log::info!("Audio output initialized successfully");
                    break (_stream, sink);
                }
                Err(e) => {
                    log::error!("Failed to create audio sink: {}", e);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            },
            Err(e) => {
                log::error!("Failed to get audio output: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    };

    // Keep _stream alive
    let _stream = stream;

    let mut sample_count = 0u64;
    let mut waiting_for_new_samples = false;

    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(samples) => {
                if samples.is_empty() {
                    log::info!("Audio: Received clear signal, clearing sink");
                    sink.clear();
                    sink.play();
                    waiting_for_new_samples = true;
                } else {
                    sample_count += 1;
                    if sample_count % 10 == 1 {
                        log::debug!(
                            "Audio: Received samples batch #{}, {} samples, sink empty: {}",
                            sample_count,
                            samples.len(),
                            sink.empty()
                        );
                    }
                    let buffer = SamplesBuffer::new(2, 48000, samples);
                    sink.append(buffer);

                    if waiting_for_new_samples {
                        let _ = playback_notify.send(());
                        waiting_for_new_samples = false;
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                log::info!("Audio thread: channel disconnected, exiting");
                break;
            }
        }
    }

    Ok(())
}

/// Handles HLS streaming and segment fetching
pub async fn run_stream_loop(
    client: Arc<NhkRadioClient>,
    stream_data: StreamData,
    mut channel_rx: watch::Receiver<ChannelKind>,
    audio_tx: std::sync::mpsc::Sender<Vec<i16>>,
) -> Result<()> {
    let mut current_channel = *channel_rx.borrow_and_update();
    let mut seen_segments: HashSet<String> = HashSet::new();
    let mut resolved_urls: HashMap<ChannelKind, String> = HashMap::new();

    loop {
        // Check for channel change
        if channel_rx.has_changed().unwrap_or(false) {
            let new_channel = *channel_rx.borrow_and_update();
            if new_channel != current_channel {
                log::info!(
                    "Channel changed from {:?} to {:?}",
                    current_channel,
                    new_channel
                );
                current_channel = new_channel;
                seen_segments.clear();
                let _ = audio_tx.send(vec![]);
            }
        }

        let m3u8_url = current_channel.get_url(&stream_data);
        log::debug!(
            "Fetching playlist for channel {:?}: {}",
            current_channel,
            m3u8_url
        );

        // Resolve master playlist if needed (cache the result)
        let actual_url = if let Some(url) = resolved_urls.get(&current_channel) {
            url.clone()
        } else {
            match resolve_master_playlist(&client, &m3u8_url).await {
                Ok(url) => {
                    log::info!("Resolved playlist URL for {:?}: {}", current_channel, url);
                    resolved_urls.insert(current_channel, url.clone());
                    url
                }
                Err(e) => {
                    log::error!("Failed to resolve master playlist: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            }
        };

        let playlist_content = match client.fetch_m3u8(&actual_url).await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to fetch playlist: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let segments = match parse_m3u8(&playlist_content, &actual_url) {
            Ok(segs) => segs,
            Err(e) => {
                log::error!("Failed to parse playlist: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
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

        let mut channel_changed = false;
        for segment in segments {
            if channel_rx.has_changed().unwrap_or(false) {
                channel_changed = true;
                break;
            }

            if seen_segments.contains(&segment.url) {
                continue;
            }
            seen_segments.insert(segment.url.clone());

            let mut data = match client.fetch_segment(&segment.url).await {
                Ok(d) => d,
                Err(e) => {
                    log::error!("Failed to fetch segment: {}", e);
                    continue;
                }
            };

            if let Some(ref k) = key {
                data = match decrypt_segment(&data, k, segment.iv.as_deref(), segment.seq_no) {
                    Ok(d) => d,
                    Err(e) => {
                        log::error!("Failed to decrypt: {}", e);
                        continue;
                    }
                };
            }

            match decode_aac_to_pcm(&data) {
                Ok(pcm_samples) if !pcm_samples.is_empty() => {
                    let _ = audio_tx.send(pcm_samples);
                }
                Ok(_) => {}
                Err(e) => {
                    log::debug!("Failed to decode AAC: {}", e);
                }
            }
        }

        if channel_changed {
            continue;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn resolve_master_playlist(client: &NhkRadioClient, m3u8_url: &str) -> Result<String> {
    let playlist_content = client.fetch_m3u8(m3u8_url).await?;

    match parse_m3u8(&playlist_content, m3u8_url) {
        Ok(_) => Ok(m3u8_url.to_string()),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Master playlist detected") {
                if let Some(variant_url) = error_msg.split("Variant URL: ").nth(1) {
                    log::info!("Detected master playlist, using variant: {}", variant_url);
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
