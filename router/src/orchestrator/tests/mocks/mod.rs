pub mod clock;

use bytes::Bytes;
use commonware_avs_core::bn254::PublicKey;
use commonware_p2p::{Receiver, Recipients, Sender};
use std::fmt;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub struct MockP2pError;

impl fmt::Display for MockP2pError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mock p2p error")
    }
}

impl std::error::Error for MockP2pError {}

/// Minimal no-op sender that satisfies the `Sender` trait bounds.
#[derive(Clone, Debug)]
pub struct MockSender;

impl MockSender {
    pub fn new() -> Self {
        Self
    }
}

impl Sender for MockSender {
    type Error = MockP2pError;
    type PublicKey = PublicKey;

    async fn send(
        &mut self,
        _recipients: Recipients<PublicKey>,
        _message: Bytes,
        _priority: bool,
    ) -> Result<Vec<PublicKey>, MockP2pError> {
        Ok(vec![])
    }
}

/// Channel-backed receiver that lets tests inject messages into the run loop.
pub struct MockReceiver {
    rx: UnboundedReceiver<(PublicKey, Bytes)>,
}

impl MockReceiver {
    pub fn new(rx: UnboundedReceiver<(PublicKey, Bytes)>) -> Self {
        Self { rx }
    }
}

impl fmt::Debug for MockReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockReceiver")
    }
}

impl Receiver for MockReceiver {
    type Error = MockP2pError;
    type PublicKey = PublicKey;

    async fn recv(&mut self) -> Result<(PublicKey, Bytes), MockP2pError> {
        self.rx.recv().await.ok_or(MockP2pError)
    }
}
