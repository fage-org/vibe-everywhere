use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    path::Path,
};

use thiserror::Error;
use vibe_agent::encryption::{
    ContentKeyPair, EncryptionError, decode_base64, derive_content_key_pair, encode_base64,
};

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum CredentialEncryption {
    Legacy {
        secret: [u8; 32],
        content_key_pair: ContentKeyPair,
    },
    DataKey {
        public_key: [u8; 32],
        machine_key: [u8; 32],
    },
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub token: String,
    pub encryption: CredentialEncryption,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StoredCredentials {
    token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encryption: Option<StoredDataKeyEncryption>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredDataKeyEncryption {
    public_key: String,
    machine_key: String,
}

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("Not authenticated. Run `vibe auth login` first.")]
    NotAuthenticated,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Encryption(#[from] EncryptionError),
}

pub fn read_credentials(config: &Config) -> Option<Credentials> {
    let raw = fs::read_to_string(&config.credential_path).ok()?;
    let parsed: StoredCredentials = serde_json::from_str(&raw).ok()?;
    if let Some(secret) = parsed.secret {
        let secret = decode_base64(&secret).ok()?;
        let secret: [u8; 32] = secret.try_into().ok()?;
        let content_key_pair = derive_content_key_pair(&secret);
        return Some(Credentials {
            token: parsed.token,
            encryption: CredentialEncryption::Legacy {
                secret,
                content_key_pair,
            },
        });
    }
    let encryption = parsed.encryption?;
    let public_key = decode_base64(&encryption.public_key).ok()?;
    let machine_key = decode_base64(&encryption.machine_key).ok()?;
    Some(Credentials {
        token: parsed.token,
        encryption: CredentialEncryption::DataKey {
            public_key: public_key.try_into().ok()?,
            machine_key: machine_key.try_into().ok()?,
        },
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
    write_credentials_legacy(config, token, secret)
}

pub fn write_credentials_legacy(
    config: &Config,
    token: impl Into<String>,
    secret: [u8; 32],
) -> Result<(), CredentialsError> {
    ensure_parent_dir(&config.credential_path)?;

    let payload = StoredCredentials {
        token: token.into(),
        secret: Some(encode_base64(&secret)),
        encryption: None,
    };
    let encoded = serde_json::to_string(&payload)?;
    write_secure_file(&config.credential_path, encoded.as_bytes())?;
    Ok(())
}

pub fn write_credentials_data_key(
    config: &Config,
    token: impl Into<String>,
    public_key: [u8; 32],
    machine_key: [u8; 32],
) -> Result<(), CredentialsError> {
    ensure_parent_dir(&config.credential_path)?;

    let payload = StoredCredentials {
        token: token.into(),
        secret: None,
        encryption: Some(StoredDataKeyEncryption {
            public_key: encode_base64(&public_key),
            machine_key: encode_base64(&machine_key),
        }),
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

impl Credentials {
    pub fn public_key(&self) -> [u8; 32] {
        match &self.encryption {
            CredentialEncryption::Legacy {
                content_key_pair, ..
            } => content_key_pair.public_key,
            CredentialEncryption::DataKey { public_key, .. } => *public_key,
        }
    }

    pub fn legacy_secret(&self) -> Option<[u8; 32]> {
        match &self.encryption {
            CredentialEncryption::Legacy { secret, .. } => Some(*secret),
            CredentialEncryption::DataKey { .. } => None,
        }
    }

    pub fn machine_key(&self) -> [u8; 32] {
        match &self.encryption {
            CredentialEncryption::Legacy { secret, .. } => *secret,
            CredentialEncryption::DataKey { machine_key, .. } => *machine_key,
        }
    }

    pub fn as_legacy_agent_credentials(&self) -> Option<vibe_agent::credentials::Credentials> {
        match &self.encryption {
            CredentialEncryption::Legacy {
                secret,
                content_key_pair,
            } => Some(vibe_agent::credentials::Credentials {
                token: self.token.clone(),
                secret: *secret,
                content_key_pair: *content_key_pair,
            }),
            CredentialEncryption::DataKey { .. } => None,
        }
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

    use super::{
        CredentialEncryption, clear_credentials, read_credentials, require_credentials,
        write_credentials, write_credentials_data_key,
    };

    fn test_config() -> (TempDir, Config) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();
        (temp_dir, config)
    }

    #[test]
    fn legacy_credentials_round_trip() {
        let (_temp_dir, config) = test_config();
        let secret = [7u8; 32];

        write_credentials(&config, "token-1", secret).unwrap();
        let credentials = read_credentials(&config).unwrap();

        assert_eq!(credentials.token, "token-1");
        assert!(matches!(
            credentials.encryption,
            CredentialEncryption::Legacy { secret: value, .. } if value == secret
        ));
    }

    #[test]
    fn data_key_credentials_round_trip() {
        let (_temp_dir, config) = test_config();
        let public_key = [3u8; 32];
        let machine_key = [4u8; 32];

        write_credentials_data_key(&config, "token-2", public_key, machine_key).unwrap();
        let credentials = read_credentials(&config).unwrap();

        assert_eq!(credentials.token, "token-2");
        assert!(matches!(
            credentials.encryption,
            CredentialEncryption::DataKey {
                public_key: pk,
                machine_key: mk
            } if pk == public_key && mk == machine_key
        ));
    }

    #[test]
    fn require_credentials_errors_when_missing() {
        let (_temp_dir, config) = test_config();

        let error = require_credentials(&config).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Not authenticated. Run `vibe auth login` first."
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
