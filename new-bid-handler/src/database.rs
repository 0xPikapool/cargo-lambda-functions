use mockall::{automock, predicate::*};
use redis::{Commands, RedisError};

#[automock]
pub trait Database {
    fn connect(&mut self);
    fn get_signer_approve_and_bal_amts(
        &mut self,
        chain_id: &str,
        verifying_contract: &str,
        signer: &str,
    ) -> Result<Option<(f64, f64)>, String>;
}

pub struct RedisDatabase {
    pub client: redis::Client,
    pub connection: Option<redis::Connection>,
}

impl Database for RedisDatabase {
    fn connect(&mut self) {
        let connection = self.client.get_connection().unwrap();
        self.connection = Some(connection);
    }

    fn get_signer_approve_and_bal_amts(
        &mut self,
        chain_id: &str,
        verifying_contract: &str,
        signer: &str,
    ) -> Result<Option<(f64, f64)>, String> {
        let signer_details_key = format!(
            "{}:{}:{}",
            chain_id,
            &verifying_contract[2..6].to_lowercase(),
            &signer[2..].to_lowercase()
        );
        let result: Result<Option<(String, String)>, RedisError> =
            self.connection.as_mut().unwrap().hget(
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
