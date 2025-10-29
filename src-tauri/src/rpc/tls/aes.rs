use aes_gcm::aead::OsRng;
use aes_gcm::{
    aead::{Aead, Nonce},
    AeadCore, Aes256Gcm, KeyInit,
};
use anyhow::Result;

pub fn aes_decode(key: &[u8], cipher_text: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    if cipher_text.len() < 12 {
        return Err(anyhow::anyhow!("cipher text too short"));
    }

    #[allow(deprecated)]
    let nonce = Nonce::<Aes256Gcm>::from_slice(&cipher_text[0..12]);
    let cipher_text = &cipher_text[12..];
    let plaintext = cipher
        .decrypt(nonce, cipher_text)
        .map_err(|e| anyhow::anyhow!("wrong decode key: {}", e))?;
    Ok(plaintext)
}

pub fn aes_encode(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let cipher_text = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut output = Vec::new();
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&cipher_text);
    Ok(output)
}

pub fn generate_aes_256_key() -> Vec<u8> {
    let key = Aes256Gcm::generate_key(OsRng);
    key.to_vec()
}
