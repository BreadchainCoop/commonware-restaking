pub mod clock;

use bytes::Bytes;
use commonware_actor::{Feedback, Unreliable};
use commonware_avs_core::bn254::PublicKey;
use commonware_p2p::{CheckedSender, LimitedSender, Receiver, Recipients};
use commonware_runtime::{IoBuf, IoBufs};
use std::fmt;
use std::time::SystemTime;
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
///
/// `Sender` is a blanket impl over `LimitedSender`, so implementing `check`
/// (returning a `CheckedSender`) is sufficient.
#[derive(Clone, Debug)]
pub struct MockSender;

impl MockSender {
    pub fn new() -> Self {
        Self
    }
}

/// `CheckedSender` returned by [`MockSender::check`]; drops every message.
#[derive(Debug)]
pub struct MockCheckedSender {
    recipients: Vec<PublicKey>,
}

impl LimitedSender for MockSender {
    type PublicKey = PublicKey;
    type Checked<'a>
        = MockCheckedSender
    where
        Self: 'a;

    fn check(
        &mut self,
        recipients: Recipients<PublicKey>,
    ) -> Result<Self::Checked<'_>, SystemTime> {
        Ok(MockCheckedSender {
            recipients: match recipients {
                Recipients::All => Vec::new(),
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

    async fn recv(&mut self) -> Result<(PublicKey, IoBuf), MockP2pError> {
        self.rx
            .recv()
            .await
            .map(|(pk, bytes)| (pk, IoBuf::from(bytes)))
            .ok_or(MockP2pError)
    }
}
