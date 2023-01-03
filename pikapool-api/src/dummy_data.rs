use crate::{auction::Auction, bid_payload::BidPayload};
use eip_712::EIP712;
use ethers::types::Address;
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
        AuctionOption::InvalidBasePrice => Auction {
            start_block: 100,
            end_block: 200,
            settlement_contract: Address::from_str("0xd2090025857B9C7B24387741f120538E928A3a59")
                .unwrap(),
            base_price: 1.25,
        },
        AuctionOption::InvalidSettlementAddress => Auction {
            start_block: 100,
            end_block: 200,
            settlement_contract: Address::from_str("0xaaa90025857B9C7B24387741f120538E928A3a59")
                .unwrap(),
            base_price: 1.25,
        },
        _ => Auction {
            start_block: 100,
            end_block: 200,
            settlement_contract: Address::from_str("0xd2090025857B9C7B24387741f120538E928A3a59")
                .unwrap(),
            base_price: 0.25,
        },
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
                "auctionContract": "0xFeebabE6b0418eC13b30aAdF129F5DcDd4f70CeA",
                "nftCount": "5",
                "basePricePerNft": "0.25",
                "tipPerNft": "0.5"
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
                        "name": "auctionContract",
                        "type": "address"
                    },
                    {
                        "name": "nftCount",
                        "type": "string"
                    },
                    {
                        "name": "basePricePerNft",
                        "type": "string"
                    },
                    {
                        "name": "tipPerNft",
                        "type": "string"
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
            "auctionContract": "{}",
            "nftCount": "5",
            "basePricePerNft": "0.25",
            "tipPerNft": "0.5"
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
                    "name": "auctionContract",
                    "type": "address"
                }},
                {{
                    "name": "nftCount",
                    "type": "string"
                }},
                {{
                    "name": "basePricePerNft",
                    "type": "string"
                }},
                {{
                    "name": "tipPerNft",
                    "type": "string"
                }}
            ]
        }}
	}}"#,
            auction_address
        ),
    };
    let typed_data = from_str::<EIP712>(json.as_str()).unwrap();

    let sender = match option {
        BidPayloadOption::BadSignerAddress => "0xakljsdfjhk",
        BidPayloadOption::SignatureDoesNotMatchSigner => {
            "0xAB2a3d9F938E13CD947Ec05AbC7FE734Df8DD820"
        }
        _ => "0x36bCaEE2F1f6C185f91608C7802f6Fc4E8bD9f1d",
    };
    let signature = match option {
        BidPayloadOption::InvalidSignature => "0xakljsdfjhk",
        _ => "0xec125943630e609fe44cafe7920232092f4413364b60ec4a21dcaf6eed01aefa668236ff37b5b37325cd580e99bbe8416937a80e65dd7a99216dbee2deafd9231b",
    };

    BidPayload {
        typed_data,
        sender: sender.to_string(),
        signature: signature.to_string(),
    }
}
