use crate::{auction::Auction, bid_payload::BidPayload};
use eip_712::EIP712;
use ethers::types::Address;
use ethers::utils::parse_ether;
use serde_json::from_str;
use std::str::FromStr;

pub enum BidPayloadOption {
    Valid,
    BadSignerAddress,
    InvalidAuctionAddress,
    InvalidSignature,
    InvalidBid,
    SignatureDoesNotMatchSigner,
}

pub enum AuctionOption {
    Valid,
    InvalidSettlementAddress,
    InvalidBasePrice,
}

pub fn new_auction(option: AuctionOption) -> Auction {
    match option {
        AuctionOption::InvalidBasePrice => Auction::new(
            Address::from_str("0xFeebabE6b0418eC13b30aAdF129F5DcDd4f70CeA").unwrap(),
            "LeafyGreens_Public_Sale".to_string(),
            100,
            200,
            Address::from_str("0xd2090025857B9C7B24387741f120538E928A3a59").unwrap(),
            parse_ether(1.25).unwrap(),
        ),
        AuctionOption::InvalidSettlementAddress => Auction::new(
            Address::from_str("0xFeebabE6b0418eC13b30aAdF129F5DcDd4f70CeA").unwrap(),
            "LeafyGreens_Public_Sale".to_string(),
            100,
            200,
            Address::from_str("0xaaa90025857B9C7B24387741f120538E928A3a59").unwrap(),
            parse_ether(1.25).unwrap(),
        ),
        _ => Auction::new(
            Address::from_str("0xFeebabE6b0418eC13b30aAdF129F5DcDd4f70CeA").unwrap(),
            "LeafyGreens_Public_Sale".to_string(),
            100,
            200,
            Address::from_str("0xd2090025857B9C7B24387741f120538E928A3a59").unwrap(),
            parse_ether(0.25).unwrap(),
        ),
    }
}

pub fn new_bid_payload(option: BidPayloadOption) -> BidPayload {
    let auction_address = match option {
        BidPayloadOption::InvalidAuctionAddress => "0x89q234r89hnbfgd",
        _ => "0xFeebabE6b0418eC13b30aAdF129F5DcDd4f70CeA",
    };

    let json = match option {
        // Typo in domain.name
        BidPayloadOption::InvalidBid => String::from(
            r#"{
            "primaryType": "Bid",
            "domain": {
                "name": "Pikapool Auctionnnnnnn",
                "version": "1",
                "chainId": "0x1",
                "verifyingContract": "0xd2090025857B9C7B24387741f120538E928A3a59"
            },
            "message": {
                "auctionAddress": "0xFeebabE6b0418eC13b30aAdF129F5DcDd4f70CeA",
                "auctionName": "LeafyGreens_Public_Sale",
                "amount": "0x5",
                "basePrice": "0x03782dace9d90000",
                "tip": "0x016345785d8a0000"
            },
            "types": {
                "EIP712Domain": [
                    {
                            "name": "name",
                            "type": "string"
                    },
                    {
                            "name": "version",
                            "type": "string"
                    },
                    {
                            "name": "chainId",
                            "type": "uint256"
                    },
                    {
                            "name": "verifyingContract",
                            "type": "address"
                    }
                ],
                "Bid": [
                    {
                        "name": "auctionName",
                        "type": "string"
                    },
                    {
                        "name": "auctionAddress",
                        "type": "address"
                    },
                    {
                        "name": "amount",
                        "type": "uint256"
                    },
                    {
                        "name": "basePrice",
                        "type": "uint256"
                    },
                    {
                        "name": "tip",
                        "type": "uint256"
                    }
                ]
            }
        }"#,
        ),
        _ => format!(
            r#"{{
        "primaryType": "Bid",
        "domain": {{
            "name": "Pikapool Auction",
            "version": "1",
            "chainId": "0x1",
            "verifyingContract": "0xd2090025857B9C7B24387741f120538E928A3a59"
        }},
        "message": {{
            "auctionName": "LeafyGreens_Public_Sale",
            "auctionAddress": "{}",
            "amount": "0x5",
            "basePrice": "0x03782dace9d90000",
            "tip": "0x016345785d8a0000"
        }},
        "types": {{
            "EIP712Domain": [
                {{
                        "name": "name",
                        "type": "string"
                }},
                {{
                        "name": "version",
                        "type": "string"
                }},
                {{
                        "name": "chainId",
                        "type": "uint256"
                }},
                {{
                        "name": "verifyingContract",
                        "type": "address"
                }}
            ],
            "Bid": [
                {{
                    "name": "auctionName",
                    "type": "string"
                }},
                {{
                    "name": "auctionAddress",
                    "type": "address"
                }},
                {{
                    "name": "amount",
                    "type": "uint256"
                }},
                {{
                    "name": "basePrice",
                    "type": "uint256"
                }},
                {{
                    "name": "tip",
                    "type": "uint256"
                }}
            ]
        }}
	}}"#,
            auction_address
        ),
    };
    let typed_data = match from_str::<EIP712>(json.as_str()) {
        Ok(typed_data) => typed_data,
        Err(e) => panic!("Error parsing typed data: {}", e),
    };

    let sender = match option {
        BidPayloadOption::BadSignerAddress => "0xakljsdfjhk",
        BidPayloadOption::SignatureDoesNotMatchSigner => {
            "0xAB2a3d9F938E13CD947Ec05AbC7FE734Df8DD820"
        }
        _ => "0x36bCaEE2F1f6C185f91608C7802f6Fc4E8bD9f1d",
    };
    let signature = match option {
        BidPayloadOption::InvalidSignature => "0xakljsdfjhk",
        _ => "0x1dd291a7d84c8e680c7132ca4812b0b54332bbc26a01afd812c790d8bacfdfd845b11da309d387c20be7a79cef88c38880db081f112a70a65c469a552685880d1b",
    };

    BidPayload {
        typed_data,
        sender: sender.to_string(),
        signature: signature.to_string(),
    }
}
