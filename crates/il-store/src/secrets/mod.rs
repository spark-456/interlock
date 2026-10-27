//! AES-256-GCM credential encryption under `IL_MASTER_KEY`.

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use il_sdk::services::store::{StoreError, StoreResult};

pub(crate) fn key_from_env() -> StoreResult<Option<[u8; 32]>> {
    let Some(value) = std::env::var_os("IL_MASTER_KEY") else { return Ok(None) };
    let value = value.to_str().ok_or_else(|| StoreError::Backend("IL_MASTER_KEY must be base64".into()))?;
    let bytes = STANDARD.decode(value).map_err(|_| StoreError::Backend("IL_MASTER_KEY must be base64".into()))?;
    let key: [u8; 32] =
        bytes.try_into().map_err(|_| StoreError::Backend("IL_MASTER_KEY must encode 32 bytes".into()))?;
    Ok(Some(key))
}

pub(crate) fn seal(key: &[u8; 32], workspace_id: &str, id: &str, plain: &[u8]) -> StoreResult<(Vec<u8>, Vec<u8>)> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| StoreError::Backend("invalid key".into()))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let aad = format!("{workspace_id}\0{id}");
    let encrypted = cipher
        .encrypt(&nonce, Payload { msg: plain, aad: aad.as_bytes() })
        .map_err(|_| StoreError::Backend("credential encryption failed".into()))?;
    Ok((nonce.to_vec(), encrypted))
}

pub(crate) fn open(
    key: &[u8; 32],
    workspace_id: &str,
    id: &str,
    nonce: &[u8],
    encrypted: &[u8],
) -> StoreResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| StoreError::Backend("invalid key".into()))?;
    if nonce.len() != 12 {
        return Err(StoreError::Backend("invalid credential nonce".into()));
    }
    let nonce = Nonce::from_slice(nonce);
    let aad = format!("{workspace_id}\0{id}");
    cipher
        .decrypt(nonce, Payload { msg: encrypted, aad: aad.as_bytes() })
        .map_err(|_| StoreError::Backend("credential decryption failed".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b01_credential_cipher_binds_workspace_and_id() {
        let key = [7; 32];
        let (nonce, ciphertext) = seal(&key, "ws_1", "cred_1", b"secret").unwrap();
        assert_ne!(ciphertext, b"secret");
        assert_eq!(open(&key, "ws_1", "cred_1", &nonce, &ciphertext).unwrap(), b"secret");
        assert!(open(&key, "ws_2", "cred_1", &nonce, &ciphertext).is_err());
    }
}
