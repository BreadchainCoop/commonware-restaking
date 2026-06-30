use crate::contributor::{AggregationInput, Contribute, ContributorBase};
use anyhow::Result;
use ark_bn254::Fr;
use commonware_avs_core::bn254::{Bn254, PrivateKey, PublicKey, Signature};
use commonware_actor::{Feedback, Unreliable};
use commonware_cryptography::Signer;
use commonware_p2p::{CheckedSender, LimitedSender, Receiver, Recipients, Sender};
use commonware_runtime::{IoBuf, IoBufs};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::time::SystemTime;

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
    type Signature = Signature;

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

// Mock implementations for testing async functionality.
//
// `Sender` is now a blanket impl over `LimitedSender`: implementing `check` (which
// returns a `CheckedSender`) is enough to get the synchronous `Sender::send`.
#[derive(Debug, Clone, Default)]
pub struct MockSender {
    peers: std::sync::Arc<[PublicKey]>,
}

/// `CheckedSender` returned by [`MockSender::check`]; drops every message.
#[derive(Debug)]
pub struct MockCheckedSender {
    recipients: Vec<PublicKey>,
}

#[derive(Debug)]
pub struct MockReceiver {
    messages: std::sync::Arc<tokio::sync::Mutex<Vec<(PublicKey, IoBuf)>>>,
}

impl MockSender {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MockReceiver {
    pub fn new() -> Self {
        Self {
            messages: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockReceiver {
    fn default() -> Self {
        Self::new()
    }
}

impl LimitedSender for MockSender {
    type PublicKey = PublicKey;
    type Checked<'a>
        = MockCheckedSender
    where
        Self: 'a;

    fn check(
        &mut self,
        recipients: Recipients<Self::PublicKey>,
    ) -> Result<Self::Checked<'_>, SystemTime> {
        Ok(MockCheckedSender {
            recipients: match recipients {
                Recipients::All => self.peers.iter().cloned().collect(),
                Recipients::Some(recipients) => recipients,
                Recipients::One(recipient) => vec![recipient],
            },
        })
    }
}

impl CheckedSender for MockCheckedSender {
    type PublicKey = PublicKey;

    fn recipients(&self) -> Vec<Self::PublicKey> {
        self.recipients.clone()
    }

    fn send(self, _message: impl Into<IoBufs> + Send, _priority: bool) -> Unreliable<Feedback> {
        Unreliable::new(Feedback::Ok)
    }
}

impl commonware_p2p::Receiver for MockReceiver {
    type Error = MockError;
    type PublicKey = PublicKey;

    async fn recv(&mut self) -> Result<(Self::PublicKey, IoBuf), Self::Error> {
        let mut messages = self.messages.lock().await;
        if messages.is_empty() {
            // Return a mock message to keep the test running
            let mock_signer = MockContributor::create_test_bn254(999);
            let mock_message = IoBuf::from(bytes::Bytes::from("mock message"));
            Ok((mock_signer.public_key(), mock_message))
        } else {
            Ok(messages.remove(0))
        }
    }
}
