use ethers::types::Address;

#[derive(Debug, Clone, PartialEq)]
pub struct Auction {
    pub id: String,
    pub address: Address,
    pub start_block: u64,
    pub end_block: u64,
    pub settlement_contract: Address,
    pub base_price: f64,
}

impl Auction {
    pub fn new(
        address: Address,
        start_block: u64,
        end_block: u64,
        settlement_contract: Address,
        base_price: f64,
    ) -> Auction {
        let id = hex::encode(address).to_lowercase() + "-" + &start_block.to_string();
        Auction {
            id,
            address,
            start_block,
            end_block,
            settlement_contract,
            base_price,
        }
    }
}
