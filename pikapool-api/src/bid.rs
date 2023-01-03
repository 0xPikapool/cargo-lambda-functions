use chrono::{DateTime, Utc};

use crate::{auction::Auction, bid_payload::BidPayload};

#[derive(Debug, Clone, PartialEq)]
pub struct Bid {
    pub payload: BidPayload,
    pub auction: Auction,
    pub received_time: DateTime<Utc>,
}

impl Bid {
    pub fn new(payload: BidPayload, received_time: DateTime<Utc>, auction: Auction) -> Bid {
        Bid {
            payload,
            received_time,
            auction,
        }
    }
}
