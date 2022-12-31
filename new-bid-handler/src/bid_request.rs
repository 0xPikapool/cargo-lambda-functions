use eip_712::EIP712;
use lazy_static::lazy_static;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;
use validator::ValidationErrors;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BidRequest {
    pub typed_data: EIP712,
    pub sender: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FieldType {
    name: String,
    r#type: String,
}

type MessageTypes = HashMap<String, Vec<FieldType>>;

lazy_static! {
    static ref EXPECTED_BID_REQUEST_TYPES_STR: String = {
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
            "Bids".to_string(),
            vec![
                FieldType {
                    name: "auctionContract".to_string(),
                    r#type: "auction".to_string(),
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
        serde_json::to_string(&types).unwrap()
    };
}

impl Validate for BidRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        // Validate types. This implicitly validates the message, as the
        // message is validated to match the types.
        let types_str = serde_json::to_string(&self.typed_data.types).unwrap();
        if types_str.eq(&EXPECTED_BID_REQUEST_TYPES_STR.to_string()) {
            return Err(ValidationErrors::new());
        };

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
