use chrono::{DateTime, Utc};

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
}
