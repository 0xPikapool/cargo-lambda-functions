use ethers::types::{Address, RecoveryMessage, Signature};
use std::str::FromStr;

pub fn verify_signature(
    signer: Address,
    hash: &String,
    signature: &String,
) -> Result<Signature, String> {
    let sig = Signature::from_str(signature).unwrap();
    let recovery_message = RecoveryMessage::from(hash.clone());
    match sig.verify(recovery_message, signer) {
        Ok(_) => Ok(sig),
        Err(_) => Err("Signature verification failed".to_string()),
    }
}
