pub mod aes;

use aes_gcm::aead::OsRng;
use anyhow::Result;
use hkdf::Hkdf;
use p256::elliptic_curve::ScalarPrimitive;
use sha2::{Digest, Sha256};

#[allow(unused)]
pub fn generate_p256_shared_secret(
    server_public_key: &[u8],
    client_secret: &[u8],
) -> Result<Vec<u8>> {
    let secret = p256::SecretKey::from_slice(&client_secret)?;
    let pubkey = p256::PublicKey::from_sec1_bytes(server_public_key)?;
    let shared = p256::ecdh::diffie_hellman(secret.to_nonzero_scalar(), pubkey.as_affine());
    Ok(shared.raw_secret_bytes().to_vec())
}

#[allow(unused)]
pub fn shared_secret_to_symmetric_secret(shares: &[u8]) -> Vec<u8> {
    let hk = Hkdf::<Sha256>::new(None, shares);
    let mut okm = [0u8; 32];
    hk.expand(&[], &mut okm)
        .expect("32 is a valid length for Sha256 to output");
    okm.to_vec()
}

pub fn generate_p256_secret() -> Result<Vec<u8>> {
    let sk = p256::SecretKey::new(ScalarPrimitive::random(&mut OsRng));
    let sk = sk.to_bytes();
    Ok(sk.to_vec())
}

pub fn get_p256_pubkey(sk: &[u8]) -> Vec<u8> {
    let sk = p256::SecretKey::from_slice(sk).unwrap();
    let pubkey = sk.public_key();
    pubkey.to_sec1_bytes().to_vec()
}

#[allow(unused)]
pub fn pubkey_to_fingerprint(pk: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(pk);
    let result = hasher.finalize();
    result.to_vec()
}
