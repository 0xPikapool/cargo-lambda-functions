use ethers::types::Address;
use ethers::types::U256;

#[derive(Debug, Clone, PartialEq)]
pub struct Auction {
    pub name: String,
    pub address: Address,
    pub start_block: u64,
    pub end_block: u64,
    pub settlement_contract: Address,
    pub base_price: U256,
}

impl Auction {
    pub fn new(
        address: Address,
        name: String,
        start_block: u64,
        end_block: u64,
        settlement_contract: Address,
        base_price: U256,
    ) -> Auction {
        Auction {
            name,
            address,
            start_block,
            end_block,
            settlement_contract,
            base_price,
        }
    }
}
