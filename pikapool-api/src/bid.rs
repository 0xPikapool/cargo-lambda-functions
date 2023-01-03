use chrono::{DateTime, Utc};

use crate::bid_payload::BidPayload;

pub struct Bid {
    payload: BidPayload,
    received_time: DateTime<Utc>,
}

impl Bid {
    pub fn new(payload: BidPayload, received_time: DateTime<Utc>) -> Bid {
        Bid {
            payload,
            received_time,
        }
    }
}
