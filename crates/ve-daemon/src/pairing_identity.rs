use std::path::Path;

use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use ve_shared::pairing_proof::PairingProof;

use crate::error::DaemonError;

type Result<T> = std::result::Result<T, DaemonError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingIdentity {
    pub installation_id: String,
    signing_key: String,
}

impl PairingIdentity {
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path).map_err(DaemonError::ConfigRead)?;
            let identity: Self = serde_json::from_str(&content).map_err(|e| {
                DaemonError::ConfigInvalid(format!("Failed to parse installation identity: {}", e))
            })?;
            return Ok(identity);
        }

        let mut seed = [0u8; 32];
        for byte in &mut seed {
            *byte = rand::random();
        }
        let signing_key = SigningKey::from_bytes(&seed);
        let installation_id = hex::encode(signing_key.verifying_key().to_bytes());
        let identity = Self {
            installation_id,
            signing_key: hex::encode(signing_key.to_bytes()),
        };
        identity.save(path)?;
        Ok(identity)
    }

    pub fn proof(&self) -> Result<PairingProof> {
        let signing_key = self.signing_key()?;
        let verifying_key = signing_key.verifying_key();
        let signature = signing_key.sign(self.installation_id.as_bytes());
        Ok(PairingProof {
            installation_id: self.installation_id.clone(),
            public_key: hex::encode(verifying_key.to_bytes()),
            signature: hex::encode(signature.to_bytes()),
        })
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(DaemonError::ConfigRead)?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            DaemonError::ConfigInvalid(format!("Failed to serialize installation identity: {}", e))
        })?;
        std::fs::write(path, content).map_err(DaemonError::ConfigRead)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(DaemonError::ConfigRead)?;
        }

        Ok(())
    }

    fn signing_key(&self) -> Result<SigningKey> {
        let signing_key = hex::decode(&self.signing_key).map_err(|_| {
            DaemonError::ConfigInvalid("Invalid installation signing key".to_string())
        })?;
        let signing_key: [u8; 32] = signing_key.try_into().map_err(|_| {
            DaemonError::ConfigInvalid("Invalid installation signing key length".to_string())
        })?;
        Ok(SigningKey::from_bytes(&signing_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_or_create_reuses_existing_identity() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("installation.json");

        let first = PairingIdentity::load_or_create(&path).unwrap();
        let second = PairingIdentity::load_or_create(&path).unwrap();

        assert_eq!(first.installation_id, second.installation_id);
    }

    #[test]
    fn proof_verifies() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("installation.json");
        let identity = PairingIdentity::load_or_create(&path).unwrap();

        let proof = identity.proof().unwrap();
        assert_eq!(proof.installation_id, identity.installation_id);
        assert!(proof.verify().is_ok());
    }
}
