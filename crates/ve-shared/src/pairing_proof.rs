use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingProof {
    pub installation_id: String,
    pub public_key: String,
    pub signature: String,
}

#[derive(Debug, Error)]
pub enum PairingProofError {
    #[error("pairing_proof installation_id mismatch")]
    InstallationIdMismatch,
    #[error("pairing_proof public_key is invalid")]
    InvalidPublicKey,
    #[error("pairing_proof signature is invalid")]
    InvalidSignature,
    #[error("pairing_proof signature verification failed")]
    VerificationFailed,
}

impl PairingProof {
    pub fn verify(&self) -> Result<(), PairingProofError> {
        if self.installation_id != installation_id_from_public_key(&self.public_key)? {
            return Err(PairingProofError::InstallationIdMismatch);
        }

        let public_key = decode_public_key(&self.public_key)?;
        let signature = decode_signature(&self.signature)?;

        public_key
            .verify(self.installation_id.as_bytes(), &signature)
            .map_err(|_| PairingProofError::VerificationFailed)
    }
}

pub fn installation_id_from_public_key(public_key_hex: &str) -> Result<String, PairingProofError> {
    let public_key = decode_public_key(public_key_hex)?;
    Ok(hex::encode(public_key.to_bytes()))
}

fn decode_public_key(public_key_hex: &str) -> Result<VerifyingKey, PairingProofError> {
    let public_key =
        hex::decode(public_key_hex).map_err(|_| PairingProofError::InvalidPublicKey)?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| PairingProofError::InvalidPublicKey)?;
    VerifyingKey::from_bytes(&public_key).map_err(|_| PairingProofError::InvalidPublicKey)
}

fn decode_signature(signature_hex: &str) -> Result<Signature, PairingProofError> {
    let signature = hex::decode(signature_hex).map_err(|_| PairingProofError::InvalidSignature)?;
    let signature: [u8; 64] = signature
        .try_into()
        .map_err(|_| PairingProofError::InvalidSignature)?;
    Ok(Signature::from_bytes(&signature))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn verify_accepts_valid_signature() {
        let signing_key = SigningKey::from_bytes(&[7; 32]);
        let verifying_key = signing_key.verifying_key();
        let installation_id = hex::encode(verifying_key.to_bytes());
        let signature = signing_key.sign(installation_id.as_bytes());
        let proof = PairingProof {
            installation_id,
            public_key: hex::encode(verifying_key.to_bytes()),
            signature: hex::encode(signature.to_bytes()),
        };

        assert!(proof.verify().is_ok());
    }

    #[test]
    fn verify_rejects_mismatched_installation_id() {
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let verifying_key = signing_key.verifying_key();
        let signature = signing_key.sign(b"wrong-installation");
        let proof = PairingProof {
            installation_id: "wrong-installation".to_string(),
            public_key: hex::encode(verifying_key.to_bytes()),
            signature: hex::encode(signature.to_bytes()),
        };

        assert!(matches!(
            proof.verify(),
            Err(PairingProofError::InstallationIdMismatch)
        ));
    }
}
