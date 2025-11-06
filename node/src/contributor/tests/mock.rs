use crate::contributor::{AggregationInput, Contribute, ContributorBase};
use anyhow::Result;
use ark_bn254::Fr;
use bn254::{Bn254, PrivateKey, PublicKey, Signature as Bn254Signature};
use commonware_cryptography::Signer;
use commonware_p2p::{Receiver, Sender};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;

/// Mock contributor for testing the trait implementations
pub struct MockContributor {
    pub orchestrator: PublicKey,
    pub signer: Bn254,
    pub me: usize,
    pub contributors: Vec<PublicKey>,
    pub ordered_contributors: HashMap<PublicKey, usize>,
    pub aggregation_data: Option<AggregationInput>,
}

impl ContributorBase for MockContributor {
    type PublicKey = PublicKey;
    type Signer = Bn254;
    type Signature = Bn254Signature;

    fn is_orchestrator(&self, sender: &Self::PublicKey) -> bool {
        &self.orchestrator == sender
    }

    fn get_contributor_index(&self, public_key: &Self::PublicKey) -> Option<&usize> {
        self.ordered_contributors.get(public_key)
    }
}

impl Contribute for MockContributor {
    type AggregationInput = AggregationInput;

    fn new(
        orchestrator: PublicKey,
        signer: Bn254,
        mut contributors: Vec<PublicKey>,
        aggregation_data: Option<AggregationInput>,
    ) -> Self {
        contributors.sort();
        let mut ordered_contributors = HashMap::new();
        for (idx, contributor) in contributors.iter().enumerate() {
            ordered_contributors.insert(contributor.clone(), idx);
        }
        let me = *ordered_contributors.get(&signer.public_key()).unwrap();

        Self {
            orchestrator,
            signer,
            me,
            contributors,
            ordered_contributors,
            aggregation_data,
        }
    }

    async fn run<S, R>(self, _sender: S, _receiver: R) -> Result<()>
    where
        S: Sender,
        R: Receiver<PublicKey = PublicKey>,
    {
        // Mock implementation - just return success
        Ok(())
    }
}

impl MockContributor {
    /// Helper function to create Bn254 instances for testing using fixed values
    pub fn create_test_bn254(seed: u64) -> Bn254 {
        let fr = Fr::from(seed);
        let private_key = PrivateKey::from(fr);
        Bn254::new(private_key).expect("Failed to create Bn254 from private key")
    }

    /// Create a mock contributor with test data
    pub fn new_test_contributor() -> Self {
        let signer = Self::create_test_bn254(1);
        let orchestrator = Self::create_test_bn254(2);
        let contributor1 = Self::create_test_bn254(3);
        let contributor2 = Self::create_test_bn254(4);

        let contributors = vec![
            signer.public_key(),
            orchestrator.public_key(),
            contributor1.public_key(),
            contributor2.public_key(),
        ];

        let aggregation_input = AggregationInput::new(3, HashMap::new());

        Self::new(
            orchestrator.public_key(),
            signer,
            contributors,
            Some(aggregation_input),
        )
    }

    /// Create a mock contributor without aggregation data
    pub fn new_simple_contributor() -> Self {
        let signer = Self::create_test_bn254(5);
        let orchestrator = Self::create_test_bn254(6);
        let contributors = vec![signer.public_key(), orchestrator.public_key()];

        Self::new(orchestrator.public_key(), signer, contributors, None)
    }
}

// Custom error type for testing
#[derive(Debug)]
pub struct MockError(String);

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockError: {}", self.0)
    }
}

impl StdError for MockError {}

// Mock implementations for testing async functionality
#[derive(Debug, Clone)]
pub struct MockSender {
    sent_messages: std::sync::Arc<tokio::sync::Mutex<Vec<(String, bytes::Bytes, bool)>>>,
}

#[derive(Debug)]
pub struct MockReceiver {
    messages: std::sync::Arc<tokio::sync::Mutex<Vec<(PublicKey, bytes::Bytes)>>>,
}

impl MockSender {
    pub fn new() -> Self {
        Self {
            sent_messages: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
}

impl MockReceiver {
    pub fn new() -> Self {
        Self {
            messages: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
}

impl commonware_p2p::Sender for MockSender {
    type Error = MockError;
    type PublicKey = PublicKey;

    async fn send(
        &mut self,
        _recipients: commonware_p2p::Recipients<Self::PublicKey>,
        message: bytes::Bytes,
        reliable: bool,
    ) -> Result<Vec<Self::PublicKey>, Self::Error> {
        let mut messages = self.sent_messages.lock().await;
        messages.push(("mock_recipients".to_string(), message, reliable));
        Ok(vec![]) // Return empty vector as required by the trait
    }
}

impl commonware_p2p::Receiver for MockReceiver {
    type Error = MockError;
    type PublicKey = PublicKey;

    async fn recv(&mut self) -> Result<(Self::PublicKey, bytes::Bytes), Self::Error> {
        let mut messages = self.messages.lock().await;
        if messages.is_empty() {
            // Return a mock message to keep the test running
            let mock_signer = MockContributor::create_test_bn254(999);
            let mock_message = bytes::Bytes::from("mock message");
            Ok((mock_signer.public_key(), mock_message))
        } else {
            Ok(messages.remove(0))
        }
    }
}
