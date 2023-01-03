use eip_712::{FieldType, MessageTypes, EIP712};
use lazy_static::lazy_static;
use serde;
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator::ValidationErrors;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BidPayload {
    pub typed_data: EIP712,
    pub sender: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BidValues {
    pub auction_contract: String,
    pub nft_count: u128,
    pub base_price_per_nft: f64,
    pub tip_per_nft: f64,
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
    pub fn get_values(&self) -> BidValues {
        let message = self.typed_data.message.as_object().unwrap();
        let auction_contract = message.get("auctionContract").unwrap().as_str().unwrap();
        let nft_count = message.get("nftCount").unwrap().as_str().unwrap();
        let base_price_per_nft = message.get("basePricePerNft").unwrap().as_str().unwrap();
        let tip_per_nft = message.get("tipPerNft").unwrap().as_str().unwrap();

        BidValues {
            auction_contract: auction_contract.to_string(),
            nft_count: nft_count.parse().unwrap(),
            base_price_per_nft: base_price_per_nft.parse().unwrap(),
            tip_per_nft: tip_per_nft.parse().unwrap(),
        }
    }

    pub fn get_bid_cost(&self) -> f64 {
        let values = self.get_values();
        values.nft_count as f64 * (values.base_price_per_nft + values.tip_per_nft)
    }
}
