use ark_bn254::Fr;
use bn254::{Bn254, PrivateKey};
use std::str::FromStr;

/// Helper function to create a test signer for testing purposes.
///
/// This function creates a deterministic test signer using a fixed
/// private key. It's useful for testing scenarios where a consistent
/// signer is needed.
///
/// # Returns
/// * `Bn254` - A test signer instance
pub fn create_test_signer() -> Bn254 {
    // Create a deterministic test signer using a simple approach
    // For testing purposes, we'll use a fixed private key
    let fr = Fr::from_str("1234567890").expect("Failed to create test Fr");
    let private_key = PrivateKey::from(fr);
    Bn254::new(private_key).expect("Failed to create test signer")
}
