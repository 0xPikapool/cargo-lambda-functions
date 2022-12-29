use crate::core::BidRequest;
use eip_712::EIP712;
use serde_json::from_str;

pub fn get_dummy_bid_request() -> BidRequest {
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

    BidRequest {
        typed_data,
        sender: "0x0000000000000000000000000000000000000000".to_string(),
        signature: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}
