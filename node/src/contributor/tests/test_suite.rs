use super::mock::{MockContributor, MockReceiver, MockSender};
use crate::contributor::{AggregationInput, Contribute, ContributorBase};
use ark_bn254::Fr;
use bn254::{Bn254, PrivateKey};
use commonware_cryptography::Signer;
use std::collections::HashMap;

// Helper function to create Bn254 instances for testing using fixed values
fn create_test_bn254(seed: u64) -> Bn254 {
    let fr = Fr::from(seed);
    let private_key = PrivateKey::from(fr);
    Bn254::new(private_key).expect("Failed to create Bn254 from private key")
}

#[cfg(test)]
mod contributor_base_tests {
    use super::*;

    #[test]
    fn test_is_orchestrator() {
        let contributor = MockContributor::new_test_contributor();

        // Test with orchestrator's public key
        assert!(contributor.is_orchestrator(&contributor.orchestrator));

        // Test with non-orchestrator's public key
        assert!(!contributor.is_orchestrator(&contributor.signer.public_key()));
    }

    #[test]
    fn test_get_contributor_index() {
        let contributor = MockContributor::new_test_contributor();

        // Test with existing contributor
        let index = contributor.get_contributor_index(&contributor.signer.public_key());
        assert!(index.is_some());
        assert_eq!(*index.unwrap(), contributor.me);

        // Test with non-existent contributor
        let random_signer = create_test_bn254(999);
        let index = contributor.get_contributor_index(&random_signer.public_key());
        assert!(index.is_none());
    }

    #[test]
    fn test_get_contributor_index_with_simple_contributor() {
        let contributor = MockContributor::new_simple_contributor();

        // Test with existing contributor
        let index = contributor.get_contributor_index(&contributor.signer.public_key());
        assert!(index.is_some());

        // Test with orchestrator
        let index = contributor.get_contributor_index(&contributor.orchestrator);
        assert!(index.is_some());
    }
}

#[cfg(test)]
mod contribute_tests {
    use super::*;

    #[test]
    fn test_new_with_aggregation_data() {
        let signer = create_test_bn254(10);
        let orchestrator = create_test_bn254(11);
        let contributor1 = create_test_bn254(12);

        let contributors = vec![
            signer.public_key(),
            orchestrator.public_key(),
            contributor1.public_key(),
        ];

        let aggregation_input = AggregationInput::new(2, HashMap::new());

        let contributor = MockContributor::new(
            orchestrator.public_key(),
            signer,
            contributors,
            Some(aggregation_input),
        );

        assert_eq!(contributor.orchestrator, orchestrator.public_key());
        assert_eq!(contributor.contributors.len(), 3);
        assert!(contributor.aggregation_data.is_some());
    }

    #[test]
    fn test_new_without_aggregation_data() {
        let signer = create_test_bn254(20);
        let orchestrator = create_test_bn254(21);
        let contributors = vec![signer.public_key(), orchestrator.public_key()];

        let contributor =
            MockContributor::new(orchestrator.public_key(), signer, contributors, None);

        assert_eq!(contributor.orchestrator, orchestrator.public_key());
        assert_eq!(contributor.contributors.len(), 2);
        assert!(contributor.aggregation_data.is_none());
    }

    #[test]
    fn test_contributors_are_sorted() {
        let signer = create_test_bn254(30);
        let orchestrator = create_test_bn254(31);
        let contributor1 = create_test_bn254(32);
        let contributor2 = create_test_bn254(33);

        // Create contributors in unsorted order
        let contributors = vec![
            contributor2.public_key(),
            signer.public_key(),
            contributor1.public_key(),
            orchestrator.public_key(),
        ];

        let contributor =
            MockContributor::new(orchestrator.public_key(), signer, contributors, None);

        // Verify contributors are sorted
        let mut sorted_contributors = contributor.contributors.clone();
        sorted_contributors.sort();
        assert_eq!(contributor.contributors, sorted_contributors);
    }

    #[test]
    fn test_me_index_calculation() {
        let signer = create_test_bn254(40);
        let orchestrator = create_test_bn254(41);
        let contributor1 = create_test_bn254(42);

        let signer_pubkey = signer.public_key();
        let contributors = vec![
            signer_pubkey.clone(),
            orchestrator.public_key(),
            contributor1.public_key(),
        ];

        let contributor =
            MockContributor::new(orchestrator.public_key(), signer, contributors, None);

        // Verify that me index corresponds to the signer's position in sorted contributors
        let signer_index = contributor.get_contributor_index(&signer_pubkey).unwrap();
        assert_eq!(contributor.me, *signer_index);
    }

    #[tokio::test]
    async fn test_run_method() {
        let contributor = MockContributor::new_test_contributor();

        // Create mock sender and receiver
        let sender = MockSender::new();
        let receiver = MockReceiver::new();

        // Test that the run method completes successfully
        let result = contributor.run(sender, receiver).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod aggregation_input_tests {
    use super::*;

    #[test]
    fn test_aggregation_input_creation() {
        let threshold = 3;
        let g1_map = HashMap::new();

        let aggregation_input = AggregationInput::new(threshold, g1_map);

        assert_eq!(aggregation_input.threshold(), threshold);
        assert!(aggregation_input.g1_map().is_empty());
    }

    #[test]
    fn test_aggregation_input_with_g1_map() {
        let threshold = 2;
        let mut g1_map = HashMap::new();
        let signer = create_test_bn254(50);
        // Create a simple G1 key for testing (using default coordinates)
        let g1_key = bn254::G1PublicKey::create_from_g1_coordinates("0", "0").unwrap();
        g1_map.insert(signer.public_key(), g1_key);

        let aggregation_input = AggregationInput::new(threshold, g1_map);

        assert_eq!(aggregation_input.threshold(), threshold);
        assert_eq!(aggregation_input.g1_map().len(), 1);
        assert!(
            aggregation_input
                .g1_map()
                .contains_key(&signer.public_key())
        );
    }
}
