use eip_712::{FieldType, MessageTypes, EIP712};
use ethers::types::U256;
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
    pub auction_name: String,
    pub auction_address: String,
    pub amount: U256,
    pub base_price: U256,
    pub tip: U256,
}

impl ParsedValues {
    pub fn get_bid_cost(&self) -> U256 {
        self.amount * (self.base_price + self.tip)
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
                    name: "auctionName".to_string(),
                    r#type: "string".to_string(),
                },
                FieldType {
                    name: "auctionAddress".to_string(),
                    r#type: "address".to_string(),
                },
                FieldType {
                    name: "amount".to_string(),
                    r#type: "uint256".to_string(),
                },
                FieldType {
                    name: "basePrice".to_string(),
                    r#type: "uint256".to_string(),
                },
                FieldType {
                    name: "tip".to_string(),
                    r#type: "uint256".to_string(),
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
        let auction_name = message
            .get("auctionName")
            .ok_or("auctionName parsing error")?
            .as_str()
            .ok_or("auctionName parsing error")?;

        let auction_address = message
            .get("auctionAddress")
            .ok_or("auctionAddress parsing error")?
            .as_str()
            .ok_or("auctionAddress parsing error")?;
        let amount = message
            .get("amount")
            .ok_or("amount parsing error")?
            .as_str()
            .ok_or("amount parsing error")?
            .parse::<U256>()
            .map_err(|e| format!("amount parsing error: {}", e.to_string()))?;
        let base_price = message
            .get("basePrice")
            .ok_or("basePrice parsing error")?
            .as_str()
            .ok_or("basePrice parsing error")?
            .parse::<U256>()
            .map_err(|e| format!("basePrice parsing error: {}", e.to_string()))?;
        let tip = message
            .get("tip")
            .ok_or("tip parsing error")?
            .as_str()
            .ok_or("tip parsing error")?
            .parse::<U256>()
            .map_err(|e| format!("tip parsing error: {}", e.to_string()))?;

        let parsed_values = ParsedValues {
            auction_name: auction_name.to_string(),
            auction_address: auction_address.to_string(),
            amount,
            base_price,
            tip,
        };
        Ok(parsed_values)
    }
}
