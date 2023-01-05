use chrono::{DateTime, Utc};
use hex;
use sha2::{Digest, Sha256};

use crate::{
    auction::Auction,
    bid_payload::{BidPayload, ParsedValues},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Bid {
    pub payload: BidPayload,
    pub parsed_values: ParsedValues,
    pub auction: Auction,
    pub received_time: DateTime<Utc>,
}

impl Bid {
    pub fn new(
        payload: BidPayload,
        parsed_values: ParsedValues,
        received_time: DateTime<Utc>,
        auction: Auction,
    ) -> Bid {
        Bid {
            payload,
            parsed_values,
            received_time,
            auction,
        }
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.payload.sender);
        hasher.update(&self.payload.signature);
        hasher.update(&self.received_time.to_rfc3339());
        hex::encode(hasher.finalize())
    }
}
