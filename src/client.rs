use anyhow::Result;
use reqwest::Client;

use crate::types::{RadiruConfig, Root};

const CONFIG_WEB_URL: &str = "https://www.nhk.or.jp/radio/config/config_web.xml";

pub struct NhkRadioClient {
    client: Client,
}

impl NhkRadioClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch NHK Radio configuration
    pub async fn fetch_config(&self) -> Result<RadiruConfig> {
        let response = self.client.get(CONFIG_WEB_URL).send().await?;
        let text = response.text().await?;
        let config: RadiruConfig = serde_xml_rs::de::from_str(&text)?;
        Ok(config)
    }

    /// Fetch program information
    pub async fn fetch_program(&self, program_url: &str) -> Result<Root> {
        let response = self.client.get(program_url).send().await?;
        let text = response.text().await?;
        let program: Root = serde_json::from_str(&text).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse JSON: {}. Response: {}",
                e,
                &text[..text.len().min(500)]
            )
        })?;
        Ok(program)
    }

    /// Fetch M3U8 playlist content
    pub async fn fetch_m3u8(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }

    /// Fetch decryption key
    pub async fn fetch_key(&self, key_url: &str) -> Result<Vec<u8>> {
        let response = self.client.get(key_url).send().await?;
        let bytes = response.bytes().await?;
        if bytes.len() != 16 {
            anyhow::bail!("Invalid key length: expected 16, got {}", bytes.len());
        }
        Ok(bytes.to_vec())
    }

    /// Fetch segment data
    pub async fn fetch_segment(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

impl Default for NhkRadioClient {
    fn default() -> Self {
        Self::new()
    }
}
