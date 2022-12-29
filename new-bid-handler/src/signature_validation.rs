use ethers::types::Signature;
use std::str::FromStr;

pub fn verify_signature(
    signer: &String,
    hash: &String,
    signature: &String,
) -> Result<Signature, String> {
    // TODO: Implement
    let sig = Signature::from_str(signature).unwrap();
    Ok(sig)
}
