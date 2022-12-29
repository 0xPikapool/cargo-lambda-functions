use crate::core::BidRequest;
use eip_712::EIP712;
use serde_json::from_str;

pub enum Option {
    Valid,
    BadSignerAddress,
    InvalidSignature,
    SignatureDoesNotMatchSigner,
}

pub fn new_bid_request(option: Option) -> BidRequest {
    let json = r#"{
		"primaryType": "Mail",
		"domain": {
			"name": "Ether Mail",
			"version": "1",
			"chainId": "0x1",
			"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
		},
		"message": {
			"from": {
				"name": "Cow",
				"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
			},
			"to": {
				"name": "Bob",
				"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
			},
			"contents": "Hello, Bob!"
		},
		"types": {
			"EIP712Domain": [
				{ "name": "name", "type": "string" },
				{ "name": "version", "type": "string" },
				{ "name": "chainId", "type": "uint256" },
				{ "name": "verifyingContract", "type": "address" }
			],
			"Person": [
				{ "name": "name", "type": "string" },
				{ "name": "wallet", "type": "address" }
			],
			"Mail": [
				{ "name": "from", "type": "Person" },
				{ "name": "to", "type": "Person" },
				{ "name": "contents", "type": "string" }
			]
		}
	}"#;
    let typed_data = from_str::<EIP712>(json).unwrap();

    let sender = match option {
        Option::BadSignerAddress => "0xakljsdfjhk",
        Option::SignatureDoesNotMatchSigner => "0xAB2a3d9F938E13CD947Ec05AbC7FE734Df8DD820",
        _ => "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826",
    };
    let signature = match option {
        Option::InvalidSignature => "0xakljsdfjhk",
        _ => "0x4355c47d63924e8a72e509b65029052eb6c299d53a04e167c5775fd466751c9d07299936d304c153f6443dfa05f40ff007d72911b6f72307f996231605b915621c",
    };

    BidRequest {
        typed_data,
        sender: sender.to_string(),
        signature: signature.to_string(),
    }
}
