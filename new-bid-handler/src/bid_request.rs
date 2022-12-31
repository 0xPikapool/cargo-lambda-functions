use eip_712::{FieldType, MessageTypes, EIP712};
use lazy_static::lazy_static;
use serde;
use serde::{Deserialize, Serialize};
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

impl Validate for BidRequest {
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
