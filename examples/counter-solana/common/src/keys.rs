//! Key-file and router-connection-file formats for the Solana counter example.

use anyhow::{Context, Result, anyhow};
use commonware_avs_jito::bn254::{Bn254Signer, PrivateKey};
use serde::Deserialize;
use std::fs;

/// Key file: `{ "privateKey": "<64 hex chars>" }` — a big-endian BN254 Fr
/// scalar, the same byte convention as `ncn-program-core`'s `PrivKey` (and
/// the key the operator registered on-chain with).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeyConfig {
    private_key: String,
}

/// Loads a BN254 signer from a key file.
pub fn load_bn254_key(path: &str) -> Result<Bn254Signer> {
    let contents = fs::read_to_string(path).with_context(|| format!("reading key file {path}"))?;
    let config: KeyConfig = serde_json::from_str(&contents)
        .with_context(|| format!("parsing key file {path} as {{\"privateKey\": hex}}"))?;
    let raw = hex::decode(config.private_key.trim_start_matches("0x"))
        .context("privateKey is not valid hex")?;
    let key = PrivateKey::try_from(raw.as_slice())
        .map_err(|e| anyhow!("privateKey is not a valid BN254 Fr scalar: {e:?}"))?;
    Bn254Signer::new(key).ok_or_else(|| anyhow!("failed to derive public keys from privateKey"))
}

/// Router connection file the nodes load (peer of the EVM example's
/// `public_router.json`): the router's compressed G2 identity key plus its
/// dialable address.
#[derive(Debug, Deserialize)]
pub struct RouterConnection {
    /// Router's compressed G2 public key (128 hex chars, on-chain wire form).
    pub g2: String,
    /// Hostname or IP the nodes dial.
    #[serde(default = "default_address")]
    pub address: String,
    /// Port the router listens on.
    pub port: u16,
}

fn default_address() -> String {
    "localhost".to_string()
}

impl RouterConnection {
    pub fn load(path: &str) -> Result<Self> {
        let contents =
            fs::read_to_string(path).with_context(|| format!("reading router file {path}"))?;
        serde_json::from_str(&contents).with_context(|| format!("parsing router file {path}"))
    }

    /// The router's public key in the jito scheme's wire form.
    pub fn public_key(&self) -> Result<commonware_avs_jito::bn254::PublicKey> {
        let raw = hex::decode(self.g2.trim_start_matches("0x")).context("g2 is not valid hex")?;
        commonware_avs_jito::bn254::PublicKey::try_from(raw.as_slice())
            .map_err(|e| anyhow!("g2 is not a valid compressed G2 point: {e:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_cryptography::Signer as _;
    use commonware_math::algebra::Random as _;
    use commonware_utils::test_rng;

    #[test]
    fn key_file_roundtrip_with_real_key() {
        let mut rng = test_rng();
        let signer = Bn254Signer::random(&mut rng);
        let hex_key = hex::encode(signer.private_key().as_ref());
        let dir = std::env::temp_dir().join(format!("counter-solana-keys-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("key.json");
        std::fs::write(&path, format!("{{\"privateKey\": \"{hex_key}\"}}")).unwrap();

        let loaded = load_bn254_key(path.to_str().unwrap()).expect("key loads");
        assert_eq!(loaded.public_key(), signer.public_key());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn router_connection_parses_and_derives_key() {
        let mut rng = test_rng();
        let signer = Bn254Signer::random(&mut rng);
        let g2_hex = hex::encode(signer.public_key().compressed());
        let json = format!("{{\"g2\": \"{g2_hex}\", \"port\": 9017}}");
        let connection: RouterConnection = serde_json::from_str(&json).unwrap();
        assert_eq!(connection.address, "localhost");
        assert_eq!(connection.port, 9017);
        assert_eq!(connection.public_key().unwrap(), signer.public_key());
    }
}
