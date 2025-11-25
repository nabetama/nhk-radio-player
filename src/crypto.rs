use aes::Aes128;
use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use anyhow::Result;
use cbc::Decryptor;

type Aes128CbcDec = Decryptor<Aes128>;

/// Decrypt segment data using AES-128-CBC
pub fn decrypt_segment(data: &[u8], key: &[u8], iv_hex: Option<&str>, seq_no: u64) -> Result<Vec<u8>> {
    if key.len() != 16 {
        anyhow::bail!("Invalid key length: expected 16, got {}", key.len());
    }

    let iv = if let Some(iv_str) = iv_hex {
        let iv_str = iv_str.trim_start_matches("0x").trim_start_matches("0X");
        hex::decode(iv_str)?
    } else {
        let mut iv_bytes = vec![0u8; 16];
        let seq_bytes = seq_no.to_be_bytes();
        iv_bytes[8..16].copy_from_slice(&seq_bytes);
        iv_bytes
    };

    if iv.len() != 16 {
        anyhow::bail!("Invalid IV length: expected 16, got {}", iv.len());
    }

    let cipher = Aes128CbcDec::new_from_slices(key, &iv)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {:?}", e))?;

    let mut buffer = data.to_vec();
    let decrypted = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

    Ok(decrypted.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decrypt_segment() {
        let key = vec![0u8; 16];
        let data = vec![0u8; 32];
        let _result = decrypt_segment(&data, &key, None, 0);
    }
}
