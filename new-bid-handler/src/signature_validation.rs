use ethers::types::{Address, Signature, H256};
use std::str::FromStr;

pub fn verify_signature(
    signer: Address,
    hash: H256,
    signature: &String,
) -> Result<Signature, String> {
    let sig = match Signature::from_str(signature) {
        Ok(sig) => sig,
        Err(_) => return Err("Invalid signature".to_string()),
    };
    match sig.verify(hash, signer) {
        Ok(_) => Ok(sig),
        Err(_) => Err("Signature does not match signer".to_string()),
    }
}
