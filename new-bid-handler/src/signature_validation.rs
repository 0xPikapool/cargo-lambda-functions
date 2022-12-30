use ethers::types::{Address, Signature, H256};
use std::str::FromStr;

pub fn verify_signature(
    signer: Address,
    typed_data_hash_bytes: [u8; 32],
    signature: &String,
) -> Result<Signature, String> {
    let sig = match Signature::from_str(signature) {
        Ok(sig) => sig,
        Err(_) => return Err("Invalid signature".to_string()),
    };
    match sig.verify(H256(typed_data_hash_bytes), signer) {
        Ok(_) => Ok(sig),
        Err(_) => Err("Signature does not match signer".to_string()),
    }
}
