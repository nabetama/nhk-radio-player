use anyhow::Result;
use m3u8_rs::{MediaPlaylist, Playlist};
use url::Url;

use crate::types::Segment;

/// Parse M3U8 playlist and extract segment information
pub fn parse_m3u8(content: &str, base_url: &str) -> Result<Vec<Segment>> {
    let parsed = m3u8_rs::parse_playlist_res(content.as_bytes());

    match parsed {
        Ok(Playlist::MasterPlaylist(master)) => {
            if master.variants.is_empty() {
                anyhow::bail!("No variants found in master playlist");
            }
            anyhow::bail!(
                "Master playlist detected. Variant URL: {}",
                normalize_url(base_url, &master.variants[0].uri)
            );
        }
        Ok(Playlist::MediaPlaylist(media)) => parse_media_playlist(&media, base_url),
        Err(e) => anyhow::bail!("Failed to parse M3U8 playlist: {}", e),
    }
}

fn parse_media_playlist(playlist: &MediaPlaylist, base_url: &str) -> Result<Vec<Segment>> {
    let mut segments = Vec::new();

    for (idx, segment) in playlist.segments.iter().enumerate() {
        let url = normalize_url(base_url, &segment.uri);

        let (key_url, iv) = if let Some(key) = &segment.key {
            let k_url = if let Some(ref uri) = key.uri {
                if !uri.is_empty() {
                    Some(normalize_url(base_url, uri))
                } else {
                    None
                }
            } else {
                None
            };

            let iv_val = if let Some(ref iv_str) = key.iv {
                if !iv_str.is_empty() {
                    Some(iv_str.clone())
                } else {
                    None
                }
            } else {
                None
            };

            (k_url, iv_val)
        } else {
            (None, None)
        };

        segments.push(Segment {
            url,
            key_url,
            iv,
            seq_no: idx as u64,
            duration: segment.duration as f64,
        });
    }

    Ok(segments)
}

/// Normalize URL - handle relative URLs
pub fn normalize_url(base_url: &str, relative_url: &str) -> String {
    if relative_url.starts_with("http://") || relative_url.starts_with("https://") {
        return relative_url.to_string();
    }

    if relative_url.starts_with("//") {
        return format!("https:{}", relative_url);
    }

    let base = match Url::parse(base_url) {
        Ok(url) => url,
        Err(_) => return relative_url.to_string(),
    };

    if relative_url.starts_with('/') {
        if let Some(domain) = base.domain() {
            return format!("{}://{}{}", base.scheme(), domain, relative_url);
        }
    }

    match base.join(relative_url) {
        Ok(url) => url.to_string(),
        Err(_) => relative_url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        let base = "https://example.com/path/to/playlist.m3u8";

        assert_eq!(
            normalize_url(base, "https://other.com/segment.ts"),
            "https://other.com/segment.ts"
        );

        assert_eq!(
            normalize_url(base, "//cdn.example.com/segment.ts"),
            "https://cdn.example.com/segment.ts"
        );

        assert_eq!(
            normalize_url(base, "/absolute/segment.ts"),
            "https://example.com/absolute/segment.ts"
        );

        assert_eq!(
            normalize_url(base, "segment.ts"),
            "https://example.com/path/to/segment.ts"
        );
    }
}
