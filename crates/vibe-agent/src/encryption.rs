use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use dryoc::{
    classic::{
        crypto_box::{crypto_box_easy, crypto_box_keypair, crypto_box_open_easy},
        crypto_core::crypto_scalarmult_base,
        crypto_secretbox::{crypto_secretbox_easy, crypto_secretbox_open_easy},
    },
    rng::copy_randombytes,
};
use ed25519_dalek::{Signer, SigningKey};
use hmac::{Hmac, Mac};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha512};
use thiserror::Error;

type HmacSha512 = Hmac<Sha512>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyTreeState {
    pub key: [u8; 32],
    pub chain_code: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentKeyPair {
    pub public_key: [u8; 32],
    pub secret_key: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EncryptionVariant {
    #[serde(rename = "legacy")]
    Legacy,
    #[serde(rename = "dataKey")]
    DataKey,
}

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("invalid base64 data")]
    InvalidBase64,
    #[error("invalid base64url data")]
    InvalidBase64Url,
    #[error("data-encryption-key bundle uses unsupported version {0}")]
    InvalidDataEncryptionKeyVersion(u8),
    #[error("data-encryption-key bundle failed to decrypt")]
    DataEncryptionKeyDecryptFailed,
    #[error("decrypted secret had invalid length {0}")]
    InvalidSecretLength(usize),
    #[error("failed to serialize encrypted JSON payload")]
    Serialize(#[from] serde_json::Error),
    #[error("AES-GCM encryption failed")]
    AesEncrypt,
    #[error("legacy secretbox encryption failed")]
    LegacyEncrypt,
}

pub fn encode_base64(data: &[u8]) -> String {
    STANDARD.encode(data)
}

pub fn decode_base64(input: &str) -> Result<Vec<u8>, EncryptionError> {
    STANDARD
        .decode(input)
        .map_err(|_| EncryptionError::InvalidBase64)
}

pub fn encode_base64url(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

pub fn decode_base64url(input: &str) -> Result<Vec<u8>, EncryptionError> {
    URL_SAFE_NO_PAD
        .decode(input)
        .map_err(|_| EncryptionError::InvalidBase64Url)
}

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    copy_randombytes(&mut bytes);
    bytes
}

pub fn hmac_sha512(key: &[u8], data: &[u8]) -> [u8; 64] {
    let mut mac =
        <HmacSha512 as Mac>::new_from_slice(key).expect("HMAC-SHA512 accepts arbitrary key sizes");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

pub fn derive_secret_key_tree_root(seed: &[u8], usage: &str) -> KeyTreeState {
    let digest = hmac_sha512(format!("{usage} Master Seed").as_bytes(), seed);
    split_key_tree_state(&digest)
}

pub fn derive_secret_key_tree_child(chain_code: &[u8; 32], index: &str) -> KeyTreeState {
    let mut data = Vec::with_capacity(index.len() + 1);
    data.push(0);
    data.extend_from_slice(index.as_bytes());
    let digest = hmac_sha512(chain_code, &data);
    split_key_tree_state(&digest)
}

pub fn derive_key(master: &[u8], usage: &str, path: &[&str]) -> [u8; 32] {
    let mut state = derive_secret_key_tree_root(master, usage);
    for segment in path {
        state = derive_secret_key_tree_child(&state.chain_code, segment);
    }
    state.key
}

pub fn derive_content_key_pair(secret: &[u8; 32]) -> ContentKeyPair {
    let derived_seed = derive_key(secret, "Happy EnCoder", &["content"]);
    let hash = Sha512::digest(derived_seed);
    let secret_key: [u8; 32] = hash[..32].try_into().expect("sha512 output is long enough");
    let mut public_key = [0u8; 32];
    crypto_scalarmult_base(&mut public_key, &secret_key);

    ContentKeyPair {
        public_key,
        secret_key,
    }
}

pub fn encrypt_with_data_key<T: Serialize>(
    data: &T,
    data_key: &[u8; 32],
) -> Result<Vec<u8>, EncryptionError> {
    let plaintext = serde_json::to_vec(data)?;
    let nonce = random_bytes::<12>();
    let cipher = Aes256Gcm::new_from_slice(data_key).expect("AES-256 key size is fixed");
    let ciphertext = cipher
        .encrypt(GenericArray::from_slice(&nonce), plaintext.as_slice())
        .map_err(|_| EncryptionError::AesEncrypt)?;

    let mut bundle = Vec::with_capacity(1 + nonce.len() + ciphertext.len());
    bundle.push(0);
    bundle.extend_from_slice(&nonce);
    bundle.extend_from_slice(&ciphertext);
    Ok(bundle)
}

pub fn decrypt_with_data_key(bundle: &[u8], data_key: &[u8; 32]) -> Option<Value> {
    if bundle.len() < 29 || bundle.first().copied() != Some(0) {
        return None;
    }

    let nonce = GenericArray::from_slice(&bundle[1..13]);
    let ciphertext = &bundle[13..];
    let cipher = Aes256Gcm::new_from_slice(data_key).ok()?;
    let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
    serde_json::from_slice(&plaintext).ok()
}

pub fn encrypt_legacy<T: Serialize>(
    data: &T,
    secret: &[u8; 32],
) -> Result<Vec<u8>, EncryptionError> {
    let plaintext = serde_json::to_vec(data)?;
    let nonce = random_bytes::<24>();
    let mut ciphertext = vec![0u8; plaintext.len() + 16];
    crypto_secretbox_easy(&mut ciphertext, &plaintext, &nonce, secret)
        .map_err(|_| EncryptionError::LegacyEncrypt)?;
    let mut bundle = Vec::with_capacity(nonce.len() + ciphertext.len());
    bundle.extend_from_slice(&nonce);
    bundle.extend_from_slice(&ciphertext);
    Ok(bundle)
}

pub fn decrypt_legacy(bundle: &[u8], secret: &[u8; 32]) -> Option<Value> {
    if bundle.len() < 24 + 16 {
        return None;
    }

    let nonce: [u8; 24] = bundle[..24].try_into().ok()?;
    let ciphertext = &bundle[24..];
    let mut plaintext = vec![0u8; ciphertext.len().checked_sub(16)?];
    crypto_secretbox_open_easy(&mut plaintext, ciphertext, &nonce, secret).ok()?;
    serde_json::from_slice(&plaintext).ok()
}

pub fn encrypt_json<T: Serialize>(
    key: &[u8; 32],
    variant: EncryptionVariant,
    data: &T,
) -> Result<Vec<u8>, EncryptionError> {
    match variant {
        EncryptionVariant::Legacy => encrypt_legacy(data, key),
        EncryptionVariant::DataKey => encrypt_with_data_key(data, key),
    }
}

pub fn decrypt_json(key: &[u8; 32], variant: EncryptionVariant, data: &[u8]) -> Option<Value> {
    match variant {
        EncryptionVariant::Legacy => decrypt_legacy(data, key),
        EncryptionVariant::DataKey => decrypt_with_data_key(data, key),
    }
}

pub fn libsodium_encrypt_for_public_key(data: &[u8], recipient_public_key: &[u8; 32]) -> Vec<u8> {
    let (ephemeral_public_key, ephemeral_secret_key) = crypto_box_keypair();
    let nonce = random_bytes::<24>();
    let mut ciphertext = vec![0u8; data.len() + 16];
    crypto_box_easy(
        &mut ciphertext,
        data,
        &nonce,
        recipient_public_key,
        &ephemeral_secret_key,
    )
    .expect("crypto_box_easy buffer sizes are correct");

    let mut bundle = Vec::with_capacity(32 + 24 + ciphertext.len());
    bundle.extend_from_slice(&ephemeral_public_key);
    bundle.extend_from_slice(&nonce);
    bundle.extend_from_slice(&ciphertext);
    bundle
}

pub fn decrypt_box_bundle(bundle: &[u8], recipient_secret_key: &[u8; 32]) -> Option<Vec<u8>> {
    if bundle.len() < 32 + 24 + 16 {
        return None;
    }

    let sender_public_key: [u8; 32] = bundle[..32].try_into().ok()?;
    let nonce: [u8; 24] = bundle[32..56].try_into().ok()?;
    let ciphertext = &bundle[56..];
    let mut plaintext = vec![0u8; ciphertext.len().checked_sub(16)?];
    crypto_box_open_easy(
        &mut plaintext,
        ciphertext,
        &nonce,
        &sender_public_key,
        recipient_secret_key,
    )
    .ok()?;
    Some(plaintext)
}

pub fn wrap_data_encryption_key(data_key: &[u8; 32], content_public_key: &[u8; 32]) -> String {
    let encrypted = libsodium_encrypt_for_public_key(data_key, content_public_key);
    let mut wrapped = Vec::with_capacity(1 + encrypted.len());
    wrapped.push(0);
    wrapped.extend_from_slice(&encrypted);
    encode_base64(&wrapped)
}

pub fn unwrap_data_encryption_key(
    encoded: &str,
    content_secret_key: &[u8; 32],
) -> Result<[u8; 32], EncryptionError> {
    let decoded = decode_base64(encoded)?;
    let Some((version, bundle)) = decoded.split_first() else {
        return Err(EncryptionError::InvalidSecretLength(0));
    };
    if *version != 0 {
        return Err(EncryptionError::InvalidDataEncryptionKeyVersion(*version));
    }

    let decrypted = decrypt_box_bundle(bundle, content_secret_key)
        .ok_or(EncryptionError::DataEncryptionKeyDecryptFailed)?;
    decrypted
        .as_slice()
        .try_into()
        .map_err(|_| EncryptionError::InvalidSecretLength(decrypted.len()))
}

pub fn auth_challenge(secret: &[u8; 32]) -> ([u8; 32], [u8; 32], [u8; 64]) {
    let signing_key = SigningKey::from_bytes(secret);
    let verifying_key = signing_key.verifying_key();
    let challenge = random_bytes::<32>();
    let signature = signing_key.sign(&challenge).to_bytes();

    (challenge, verifying_key.to_bytes(), signature)
}

fn split_key_tree_state(digest: &[u8; 64]) -> KeyTreeState {
    KeyTreeState {
        key: digest[..32].try_into().expect("digest is long enough"),
        chain_code: digest[32..].try_into().expect("digest is long enough"),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ContentKeyPair, EncryptionVariant, auth_challenge, decode_base64, decode_base64url,
        decrypt_box_bundle, decrypt_json, derive_content_key_pair, derive_key,
        derive_secret_key_tree_root, encode_base64, encode_base64url, encrypt_json, hmac_sha512,
        libsodium_encrypt_for_public_key, random_bytes, unwrap_data_encryption_key,
        wrap_data_encryption_key,
    };

    fn to_hex(bytes: &[u8]) -> String {
        hex::encode_upper(bytes)
    }

    #[test]
    fn base64_helpers_round_trip() {
        let input = [0u8, 1, 2, 255, 128, 64];
        assert_eq!(decode_base64(&encode_base64(&input)).unwrap(), input);
        assert_eq!(decode_base64url(&encode_base64url(&input)).unwrap(), input);
    }

    #[test]
    fn hmac_sha512_has_expected_size() {
        let mac = hmac_sha512(b"key", b"data");
        assert_eq!(mac.len(), 64);
    }

    #[test]
    fn root_derivation_matches_happy_vector() {
        let root = derive_secret_key_tree_root(b"test seed", "test usage");
        assert_eq!(
            to_hex(&root.key),
            "E6E55652456F9FE47D6FF46CA3614E85B499F77E7B340FBBB1553307CEDC1E74"
        );
    }

    #[test]
    fn full_derivation_matches_happy_vector() {
        let key = derive_key(b"test seed", "test usage", &["child1", "child2"]);
        assert_eq!(
            to_hex(&key),
            "1011C097D2105D27362B987A631496BBF68B836124D1D072E9D1613C6028CF75"
        );
    }

    #[test]
    fn derive_content_key_pair_is_deterministic() {
        let secret = random_bytes::<32>();
        let first: ContentKeyPair = derive_content_key_pair(&secret);
        let second = derive_content_key_pair(&secret);

        assert_eq!(first.public_key, second.public_key);
        assert_eq!(first.secret_key, second.secret_key);
    }

    #[test]
    fn aes_payload_round_trips() {
        let key = random_bytes::<32>();
        let value = json!({"hello": "world", "nested": [1, 2, 3]});
        let encrypted = encrypt_json(&key, EncryptionVariant::DataKey, &value).unwrap();
        let decrypted = decrypt_json(&key, EncryptionVariant::DataKey, &encrypted).unwrap();

        assert_eq!(decrypted, value);
    }

    #[test]
    fn legacy_payload_round_trips() {
        let key = random_bytes::<32>();
        let value = json!({"legacy": true});
        let encrypted = encrypt_json(&key, EncryptionVariant::Legacy, &value).unwrap();
        let decrypted = decrypt_json(&key, EncryptionVariant::Legacy, &encrypted).unwrap();

        assert_eq!(decrypted, value);
    }

    #[test]
    fn box_bundle_round_trips() {
        let secret = random_bytes::<32>();
        let content_keys = derive_content_key_pair(&secret);
        let payload = vec![1u8, 2, 3, 4, 5];
        let encrypted = libsodium_encrypt_for_public_key(&payload, &content_keys.public_key);
        let decrypted = decrypt_box_bundle(&encrypted, &content_keys.secret_key).unwrap();

        assert_eq!(decrypted, payload);
    }

    #[test]
    fn wrapped_data_encryption_key_round_trips() {
        let secret = random_bytes::<32>();
        let content_keys = derive_content_key_pair(&secret);
        let data_key = random_bytes::<32>();
        let wrapped = wrap_data_encryption_key(&data_key, &content_keys.public_key);
        let unwrapped = unwrap_data_encryption_key(&wrapped, &content_keys.secret_key).unwrap();

        assert_eq!(unwrapped, data_key);
    }

    #[test]
    fn auth_challenge_has_expected_lengths() {
        let secret = random_bytes::<32>();
        let (challenge, public_key, signature) = auth_challenge(&secret);

        assert_eq!(challenge.len(), 32);
        assert_eq!(public_key.len(), 32);
        assert_eq!(signature.len(), 64);
    }
}
