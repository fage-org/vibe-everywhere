use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    path::Path,
};

use thiserror::Error;

use crate::{
    config::Config,
    encryption::{
        ContentKeyPair, EncryptionError, decode_base64, derive_content_key_pair, encode_base64,
    },
};

#[derive(Debug, Clone)]
pub struct Credentials {
    pub token: String,
    pub secret: [u8; 32],
    pub content_key_pair: ContentKeyPair,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StoredCredentials {
    token: String,
    secret: String,
}

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("Not authenticated. Run `vibe-agent auth login` first.")]
    NotAuthenticated,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Encryption(#[from] EncryptionError),
    #[error("credential secret had invalid length {0}")]
    InvalidSecretLength(usize),
}

pub fn read_credentials(config: &Config) -> Option<Credentials> {
    let raw = fs::read_to_string(&config.credential_path).ok()?;
    let parsed: StoredCredentials = serde_json::from_str(&raw).ok()?;
    let secret = decode_base64(&parsed.secret).ok()?;
    let secret: [u8; 32] = secret.try_into().ok()?;
    let content_key_pair = derive_content_key_pair(&secret);

    Some(Credentials {
        token: parsed.token,
        secret,
        content_key_pair,
    })
}

pub fn require_credentials(config: &Config) -> Result<Credentials, CredentialsError> {
    read_credentials(config).ok_or(CredentialsError::NotAuthenticated)
}

pub fn write_credentials(
    config: &Config,
    token: impl Into<String>,
    secret: [u8; 32],
) -> Result<(), CredentialsError> {
    ensure_parent_dir(&config.credential_path)?;

    let payload = StoredCredentials {
        token: token.into(),
        secret: encode_base64(&secret),
    };
    let encoded = serde_json::to_string(&payload)?;
    write_secure_file(&config.credential_path, encoded.as_bytes())?;
    Ok(())
}

pub fn clear_credentials(config: &Config) -> Result<(), CredentialsError> {
    match fs::remove_file(&config.credential_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn ensure_parent_dir(path: &Path) -> Result<(), std::io::Error> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;

        let mut builder = DirBuilder::new();
        builder.recursive(true).mode(0o700);
        builder.create(parent)?;
    }

    #[cfg(not(unix))]
    {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

fn write_secure_file(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(bytes)?;
        file.flush()?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        file.write_all(bytes)?;
        file.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::config::Config;

    use super::{clear_credentials, read_credentials, require_credentials, write_credentials};

    fn test_config() -> (TempDir, Config) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
        )
        .unwrap();
        (temp_dir, config)
    }

    #[test]
    fn credentials_round_trip() {
        let (_temp_dir, config) = test_config();
        let secret = [7u8; 32];

        write_credentials(&config, "token-1", secret).unwrap();
        let credentials = read_credentials(&config).unwrap();

        assert_eq!(credentials.token, "token-1");
        assert_eq!(credentials.secret, secret);
        assert_eq!(credentials.content_key_pair.public_key.len(), 32);
    }

    #[test]
    fn require_credentials_errors_when_missing() {
        let (_temp_dir, config) = test_config();

        let error = require_credentials(&config).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Not authenticated. Run `vibe-agent auth login` first."
        );
    }

    #[test]
    fn clear_credentials_is_idempotent() {
        let (_temp_dir, config) = test_config();
        let secret = [9u8; 32];

        write_credentials(&config, "token-1", secret).unwrap();
        clear_credentials(&config).unwrap();
        clear_credentials(&config).unwrap();

        assert!(read_credentials(&config).is_none());
    }
}
