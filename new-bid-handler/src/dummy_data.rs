use crate::{auction::Auction, bid_request::BidRequest};
use eip_712::EIP712;
use ethers::types::Address;
use serde_json::from_str;
use std::str::FromStr;

pub enum BidRequestOption {
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
        AuctionOption::Valid => Auction {
            start_block: 100,
            end_block: 200,
            settlement_contract: Address::from_str("0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC")
                .unwrap(),
            base_price: 0.25,
        },
        AuctionOption::InvalidBasePrice => Auction {
            start_block: 100,
            end_block: 200,
            settlement_contract: Address::from_str("0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC")
                .unwrap(),
            base_price: 1.25,
        },
        _ => Auction {
            start_block: 100,
            end_block: 200,
            settlement_contract: Address::from_str("0x0000000000000000000000000000000000000000")
                .unwrap(),
            base_price: 0.25,
        },
    }
}

pub fn new_bid_request(option: BidRequestOption) -> BidRequest {
    let auction_address = match option {
        BidRequestOption::InvalidAuctionAddress => "0x89q234r89hnbfgd",
        _ => "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC",
    };

    let json = match option {
        // Typo in domain.name
        BidRequestOption::InvalidBid => String::from(
            r#"{
            "primaryType": "Bid",
            "domain": {
                "name": "Pikapool Auctionnnnnnn",
                "version": "1",
                "chainId": "0x1",
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "auctionContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC",
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
            "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
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
        BidRequestOption::BadSignerAddress => "0xakljsdfjhk",
        BidRequestOption::SignatureDoesNotMatchSigner => {
            "0xAB2a3d9F938E13CD947Ec05AbC7FE734Df8DD820"
        }
        _ => "0x36bCaEE2F1f6C185f91608C7802f6Fc4E8bD9f1d",
    };
    let signature = match option {
        BidRequestOption::InvalidSignature => "0xakljsdfjhk",
        _ => "0x3a792f9eb87e3ff5134efb70995e2fe23083e6970305152cb04dd14b877f31e20f17a866799457c65f0526aa01487d7c6d24d3a4ab4a666720d7ad6b37a49a501b",
    };

    BidRequest {
        typed_data,
        sender: sender.to_string(),
        signature: signature.to_string(),
    }
}
