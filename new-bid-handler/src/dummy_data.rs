use crate::bid_request::BidRequest;
use eip_712::EIP712;
use serde_json::from_str;

pub enum Option {
    Valid,
    BadSignerAddress,
    InvalidSignature,
    InvalidBid,
    SignatureDoesNotMatchSigner,
}

pub fn new_bid_request(option: Option) -> BidRequest {
    let json = match option {
        // Typo in domain.name
        Option::InvalidBid => {
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
        }"#
        }
        _ => {
            r#"{
        "primaryType": "Bid",
        "domain": {
            "name": "Pikapool Auction",
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
	}"#
        }
    };
    let typed_data = from_str::<EIP712>(json).unwrap();

    let sender = match option {
        Option::BadSignerAddress => "0xakljsdfjhk",
        Option::SignatureDoesNotMatchSigner => "0xAB2a3d9F938E13CD947Ec05AbC7FE734Df8DD820",
        _ => "0x36bCaEE2F1f6C185f91608C7802f6Fc4E8bD9f1d",
    };
    let signature = match option {
        Option::InvalidSignature => "0xakljsdfjhk",
        _ => "0x3a792f9eb87e3ff5134efb70995e2fe23083e6970305152cb04dd14b877f31e20f17a866799457c65f0526aa01487d7c6d24d3a4ab4a666720d7ad6b37a49a501b",
    };

    BidRequest {
        typed_data,
        sender: sender.to_string(),
        signature: signature.to_string(),
    }
}
