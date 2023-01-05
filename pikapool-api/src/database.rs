use crate::bid::Bid;
use crate::utils::get_env_var;
use crate::utils::Connectable;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio_postgres::NoTls;

#[async_trait]
pub trait Database: Connectable {
    async fn insert_bid(&mut self, bid: &Bid) -> Result<String, String>;
}

pub struct RdsProvider {
    pub client: Option<tokio_postgres::Client>,
}

#[async_trait]
impl Database for RdsProvider {
    async fn insert_bid(&mut self, bid: &Bid) -> Result<String, String> {
        let client = match self.client.as_mut() {
            Some(client) => client,
            None => return Err("Failed to get postgres client".to_string()),
        };

        let now: DateTime<Utc> = Utc::now();
        let now_iso: String = now.to_rfc3339();
        let id = bid.hash();
        let query = format!(
            "
                BEGIN;
                    INSERT INTO bids
                        (auction_address, auction_name, bundle_hash, tx_hash, bid_id, signer, amount, tip_hidden, tip_revealed, status, submitted_timestamp, status_last_updated, signature)
                    VALUES('{auction_address}', '{auction_name}', NULL, NULL, '{bid_id}', '{signer}', {amount}, {tip_hidden}, NULL, 'submitted', '{submitted_timestamp_iso}', '{now_iso}', '{signature}');

                    UPDATE bids SET 
                        status = 'replaced',
                        replaced_by = '{bid_id}'
                    WHERE
                        auction_address = '{auction_address}'
                        AND auction_name = '{auction_name}'
                        AND signer = '{signer}'
                        AND status = 'submitted'
                        AND bid_id != '{bid_id}';
                COMMIT;
            ", 
                bid_id=id,
                now_iso=now_iso,
                auction_name=bid.parsed_values.auction_name,
                auction_address=hex::encode(bid.auction.address),
                signer=&bid.payload.sender[2..],
                amount=bid.parsed_values.amount,
                tip_hidden=bid.parsed_values.tip,
                submitted_timestamp_iso=bid.received_time.to_rfc3339(),
                signature=&bid.payload.signature[2..],
        );
        match client.batch_execute(&query).await {
            Ok(_) => Ok("0x".to_string() + &id),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[async_trait]
impl Connectable for RdsProvider {
    async fn is_connected(&self) -> bool {
        self.client.is_some()
    }
    async fn ping(&mut self) -> Result<(), String> {
        let client = match self.client.as_mut() {
            Some(client) => client,
            None => return Err("Failed to get postgres client".to_string()),
        };
        match client.query("SELECT 1", &[]).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
    async fn connect(&mut self) -> Result<(), String> {
        let host = get_env_var("RDS_HOST")?;
        let port = get_env_var("RDS_PORT")?;
        let user = get_env_var("RDS_USER")?;
        let password = get_env_var("RDS_PASSWORD")?;
        let dbname = get_env_var("RDS_DBNAME")?;
        let connect_string = format!(
            "host={} port={} user={} password={} dbname={}",
            host, port, user, password, dbname
        );

        let (client, connection) = match tokio_postgres::connect(&connect_string, NoTls).await {
            Ok(client) => client,
            Err(e) => return Err(e.to_string()),
        };

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        self.client = Some(client);
        Ok(())
    }
}
