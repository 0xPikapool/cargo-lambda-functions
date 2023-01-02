use ethers::types::Address;

pub struct Auction {
    pub start_block: u64,
    pub end_block: u64,
    pub settlement_contract: Address,
    pub base_price: f64,
}
