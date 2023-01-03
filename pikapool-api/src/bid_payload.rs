use eip_712::{FieldType, MessageTypes, EIP712};
use lazy_static::lazy_static;
use serde;
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator::ValidationErrors;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BidPayload {
    pub typed_data: EIP712,
    pub sender: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ParsedValues {
    pub auction_contract: String,
    pub nft_count: u128,
    pub base_price_per_nft: f64,
    pub tip_per_nft: f64,
}

impl ParsedValues {
    pub fn get_bid_cost(&self) -> f64 {
        self.nft_count as f64 * (self.base_price_per_nft + self.tip_per_nft)
    }
}

lazy_static! {
    static ref EXPECTED_BID_REQUEST_MESSAGE_TYPES: MessageTypes = {
        let mut types = MessageTypes::new();
        types.insert(
            "EIP712Domain".to_string(),
            vec![
                FieldType {
                    name: "name".to_string(),
                    r#type: "string".to_string(),
                },
                FieldType {
                    name: "version".to_string(),
                    r#type: "string".to_string(),
                },
                FieldType {
                    name: "chainId".to_string(),
                    r#type: "uint256".to_string(),
                },
                FieldType {
                    name: "verifyingContract".to_string(),
                    r#type: "address".to_string(),
                },
            ],
        );
        types.insert(
            "Bid".to_string(),
            vec![
                FieldType {
                    name: "auctionContract".to_string(),
                    r#type: "address".to_string(),
                },
                FieldType {
                    name: "nftCount".to_string(),
                    r#type: "string".to_string(),
                },
                FieldType {
                    name: "basePricePerNft".to_string(),
                    r#type: "string".to_string(),
                },
                FieldType {
                    name: "tipPerNft".to_string(),
                    r#type: "string".to_string(),
                },
            ],
        );
        types
    };
}

impl Validate for BidPayload {
    fn validate(&self) -> Result<(), ValidationErrors> {
        if self.typed_data.types != *EXPECTED_BID_REQUEST_MESSAGE_TYPES {
            return Err(ValidationErrors::new());
        }

        // Validate static domain fields
        if self.typed_data.domain.name != "Pikapool Auction" {
            return Err(ValidationErrors::new());
        };
        if self.typed_data.domain.version != "1" {
            return Err(ValidationErrors::new());
        };

        // Validate primary type
        if self.typed_data.primary_type != "Bid" {
            return Err(ValidationErrors::new());
        };

        Ok(())
    }
}

impl BidPayload {
    pub fn parse_values(&self) -> Result<ParsedValues, String> {
        //
        // AVERT YOUR EYES
        //
        let message = self
            .typed_data
            .message
            .as_object()
            .ok_or("TypedData message must be an object")?;
        let auction_contract = message
            .get("auctionContract")
            .ok_or("auctionContract parsing error")?
            .as_str()
            .ok_or("auctionContract parsing error")?;
        let nft_count = message
            .get("nftCount")
            .ok_or("nftCount parsing error")?
            .as_str()
            .ok_or("nftCount parsing error")?
            .parse::<u128>()
            .map_err(|e| format!("nftCount parsing error: {}", e.to_string()))?;
        let base_price_per_nft = message
            .get("basePricePerNft")
            .ok_or("basePricePerNft parsing error")?
            .as_str()
            .ok_or("basePricePerNft parsing error")?
            .parse::<f64>()
            .map_err(|e| format!("basePricePerNft parsing error: {}", e.to_string()))?;
        let tip_per_nft = message
            .get("tipPerNft")
            .ok_or("tipPerNft parsing error")?
            .as_str()
            .ok_or("tipPerNft parsing error")?
            .parse::<f64>()
            .map_err(|e| format!("tipPerNft parsing error: {}", e.to_string()))?;

        if tip_per_nft < 0.0 {
            return Err("tip must be greater than or equal to 0".to_string());
        }

        let parsed_values = ParsedValues {
            auction_contract: auction_contract.to_string(),
            nft_count,
            base_price_per_nft,
            tip_per_nft,
        };
        Ok(parsed_values)
    }
}
