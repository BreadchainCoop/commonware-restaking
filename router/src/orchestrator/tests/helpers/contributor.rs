use ark_bn254::Fr;
use bn254::{Bn254, G1PublicKey, PrivateKey, PublicKey};
use commonware_cryptography::Signer;
use std::collections::HashMap;
use std::str::FromStr;

/// Helper function to create test contributors for testing purposes.
///
/// This function creates a set of test contributors with their
/// corresponding G1 public key mappings. It's useful for testing
/// scenarios where multiple contributors are needed.
///
/// # Returns
/// * `(Vec<PublicKey>, HashMap<PublicKey, G1PublicKey>)` - A tuple containing:
///   - Vector of contributor public keys
///   - HashMap mapping contributor public keys to their G1 public keys
pub fn create_test_contributors() -> (Vec<PublicKey>, HashMap<PublicKey, G1PublicKey>) {
    let mut contributors = Vec::new();
    let mut g1_map = HashMap::new();

    // Create 3 test contributors
    for i in 0..3 {
        let fr = Fr::from_str(&format!("{}", i + 1000)).expect("Failed to create test Fr");
        let private_key = PrivateKey::from(fr);
        let signer = Bn254::new(private_key).expect("Failed to create contributor signer");
        let pub_key = signer.public_key();

        // Create a mock G1 public key using coordinates
        let g1_pub_key = G1PublicKey::create_from_g1_coordinates(
            &format!("{i:064x}"),
            &format!("{:064x}", i + 1),
        )
        .expect("Failed to create G1 key");

        contributors.push(pub_key.clone());
        g1_map.insert(pub_key, g1_pub_key);
    }

    (contributors, g1_map)
}
