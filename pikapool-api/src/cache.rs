use crate::utils::get_env_var;
use crate::{auction::Auction, utils::Connectable};
use async_trait::async_trait;
use ethers::types::Address;
use hex;
use redis::{Commands, RedisError};
use std::str::FromStr;

#[async_trait]
pub trait Cache: Connectable {
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

pub struct RedisCache {
    pub connection: Option<redis::Connection>,
}

impl Cache for RedisCache {
    fn get_synced_block(
        &mut self,
        chain_id: &str,
        settlement_contract: &Address,
    ) -> Result<u64, String> {
        let connection = match self.connection.as_mut() {
            Some(connection) => connection,
            None => return Err("Couldn't get redis connection".to_string()),
        };
        let key = format!(
            "{}:{}:syncedBlock",
            chain_id,
            hex::encode(settlement_contract)[..4].to_lowercase()
        );

        match connection.get(key) {
            Ok(block) => Ok(block),
            Err(err) => Err(err.to_string()),
        }
    }

    fn get_auction(
        &mut self,
        chain_id: &str,
        auction_contract: &Address,
    ) -> Result<Option<Auction>, String> {
        let connection = match self.connection.as_mut() {
            Some(connection) => connection,
            None => return Err("Failed to get redis connection".to_string()),
        };
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
                    let settlement_contract = match Address::from_str(settlement_contract.as_str())
                    {
                        Ok(address) => address,
                        Err(err) => return Err(err.to_string()),
                    };
                    Ok(Some(Auction {
                        start_block,
                        end_block,
                        settlement_contract,
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
        let connection = match self.connection.as_mut() {
            Some(connection) => connection,
            None => return Err("Failed to get redis connection".to_string()),
        };
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
                        match approve_amt.parse::<f64>() {
                            Ok(f) => f,
                            Err(err) => return Err(err.to_string()),
                        }
                    };
                    let bal_amt: f64 = match bal_amt.parse() {
                        Ok(f) => f,
                        Err(err) => return Err(err.to_string()),
                    };
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

#[async_trait]
impl Connectable for RedisCache {
    async fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    async fn ping(&mut self) -> Result<(), String> {
        let connection = match self.connection.as_mut() {
            Some(connection) => connection,
            None => return Err("Failed to get redis connection".to_string()),
        };
        match redis::cmd("PING").query::<String>(&mut *connection) {
            Ok(response) => {
                if response == "PONG" {
                    Ok(())
                } else {
                    Err("Ping returned unexpected result".to_string())
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }

    async fn connect(&mut self) -> Result<(), String> {
        let redis_url = get_env_var("REDIS_URL")?;
        let client = match redis::Client::open(redis_url) {
            Ok(client) => client,
            Err(err) => return Err(err.to_string()),
        };
        let connection = match client.get_connection() {
            Ok(connection) => connection,
            Err(err) => return Err(err.to_string()),
        };
        self.connection = Some(connection);
        match self.ping().await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
