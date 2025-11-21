use bn254::{G1PublicKey, PublicKey as PubKey};
use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error, Read, Write};
use std::collections::HashMap;

/// Input data for aggregation functionality
pub struct AggregationInput {
    threshold: usize,
    g1_map: HashMap<PubKey, G1PublicKey>,
}

impl AggregationInput {
    pub fn new(threshold: usize, g1_map: HashMap<PubKey, G1PublicKey>) -> Self {
        Self { threshold, g1_map }
    }

    pub fn threshold(&self) -> usize {
        self.threshold
    }

    pub fn g1_map(&self) -> &HashMap<PubKey, G1PublicKey> {
        &self.g1_map
    }
}

/// Internal aggregation data structure
pub struct AggregationData {
    pub threshold: usize,
    pub g1_map: HashMap<PubKey, G1PublicKey>,
    pub contributors: Vec<PubKey>,
    pub ordered_contributors: HashMap<PubKey, usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Empty;

impl Write for Empty {
    fn write(&self, _buf: &mut impl BufMut) {}
}

impl Read for Empty {
    type Cfg = ();
    fn read_cfg(_buf: &mut impl Buf, _cfg: &()) -> Result<Self, Error> {
        Ok(Empty)
    }
}

impl EncodeSize for Empty {
    fn encode_size(&self) -> usize {
        0
    }
}
