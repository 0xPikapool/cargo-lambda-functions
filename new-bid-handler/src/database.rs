use crate::auction::Auction;
use ethers::types::Address;
use hex;
use mockall::{automock, predicate::*};
use redis::{Commands, RedisError};
use std::env;
use std::str::FromStr;

#[automock]
pub trait Database {
    fn connect(&mut self);
    fn is_connected(&self) -> bool;
    fn get_signer_approve_and_bal_amts(
        &mut self,
        chain_id: &str,
        verifying_contract: &Address,
        signer: &Address,
    ) -> Result<Option<(f64, f64)>, String>;
    fn get_auction(
        &mut self,
        chain_id: &str,
        auction_contract: &Address,
    ) -> Result<Option<Auction>, String>;
    fn get_synced_block(
        &mut self,
        chain_id: &str,
        settlement_contract: &Address,
    ) -> Result<u64, String>;
}

pub struct RedisDatabase {
    pub connection: Option<redis::Connection>,
}

impl Database for RedisDatabase {
    fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    fn connect(&mut self) {
        let redis_url = env::var("REDIS_URL").unwrap();
        let client = redis::Client::open(redis_url).unwrap();
        let connection = client.get_connection().unwrap();
        self.connection = Some(connection);
    }

    fn get_synced_block(
        &mut self,
        chain_id: &str,
        settlement_contract: &Address,
    ) -> Result<u64, String> {
        let connection = self.connection.as_mut().unwrap();
        let key = format!(
            "{}:{}:syncedBlock",
            chain_id,
            hex::encode(settlement_contract)[..4].to_lowercase()
        );
        Ok(connection.get(key).unwrap())
    }

    fn get_auction(
        &mut self,
        chain_id: &str,
        auction_contract: &Address,
    ) -> Result<Option<Auction>, String> {
        let connection = self.connection.as_mut().unwrap();
        let auction_key = format!(
            "{}:auction:{}",
            chain_id,
            hex::encode(auction_contract).to_lowercase()
        );

        let result: Result<Option<(u64, u64, String, f64)>, RedisError> = connection.hget(
            &auction_key,
            &["startBlock", "endBlock", "settlementContract", "basePrice"],
        );

        match result {
            Ok(option) => match option {
                Some((start_block, end_block, settlement_contract, base_price)) => {
                    Ok(Some(Auction {
                        start_block,
                        end_block,
                        settlement_contract: Address::from_str(settlement_contract.as_str())
                            .unwrap(),
                        base_price,
                    }))
                }
                None => Ok(None),
            },
            // Response was nil means key doesn't exist -- user not approved
            Err(e) => {
                if e.to_string().contains("response was nil") {
                    Ok(None)
                } else {
                    Err(e.to_string())
                }
            }
        }
    }

    fn get_signer_approve_and_bal_amts(
        &mut self,
        chain_id: &str,
        verifying_contract: &Address,
        signer: &Address,
    ) -> Result<Option<(f64, f64)>, String> {
        let signer_details_key = format!(
            "{}:{}:{}",
            chain_id,
            &hex::encode(verifying_contract)[..4].to_lowercase(),
            &hex::encode(signer).to_lowercase()
        );
        let connection = self.connection.as_mut().unwrap();
        let result: Result<Option<(String, String)>, RedisError> = connection.hget(
            &signer_details_key,
            &["lastApproveValue", "lastBalanceValue"],
        );

        match result {
            Ok(option) => match option {
                Some((approve_amt, bal_amt)) => {
                    let approve_amt_float = if approve_amt == "GTE_U32" {
                        f64::MAX
                    } else {
                        approve_amt.parse::<f64>().unwrap()
                    };
                    let bal_amt: f64 = bal_amt.parse().unwrap();
                    Ok(Some((approve_amt_float, bal_amt)))
                }
                None => Ok(None),
            },
            // Response was nil means key doesn't exist -- user not approved
            Err(e) => {
                if e.to_string().contains("response was nil") {
                    Ok(None)
                } else {
                    Err(e.to_string())
                }
            }
        }
    }
}
