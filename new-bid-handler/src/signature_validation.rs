use ethers::types::{Address, RecoveryMessage, Signature};
use std::str::FromStr;

pub fn verify_signature(
    signer: Address,
    hash: &String,
    signature: &String,
) -> Result<Signature, String> {
    let sig = match Signature::from_str(signature) {
        Ok(sig) => sig,
        Err(_) => return Err("Invalid signature".to_string()),
    };
    let recovery_message = RecoveryMessage::from(hash.clone());
    match sig.verify(recovery_message, signer) {
        Ok(_) => Ok(sig),
        Err(_) => Err("Signature does not match signer".to_string()),
    }
}
