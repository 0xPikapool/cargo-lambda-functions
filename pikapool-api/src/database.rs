use crate::bid::Bid;
use crate::utils::get_env_var;
use crate::utils::Connectable;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio_postgres::NoTls;
use uuid::Uuid;

#[async_trait]
pub trait Database: Connectable {
    async fn insert_bid(&mut self, bid: &Bid) -> Result<(), String>;
}

pub struct RdsProvider {
    pub client: Option<tokio_postgres::Client>,
}

#[async_trait]
impl Database for RdsProvider {
    async fn insert_bid(&mut self, bid: &Bid) -> Result<(), String> {
        let client = match self.client.as_mut() {
            Some(client) => client,
            None => return Err("Failed to get postgres client".to_string()),
        };

        let now: DateTime<Utc> = Utc::now();
        let now_iso: String = now.to_rfc3339();
        let query = format!(
            "
            INSERT INTO bids
                (auction_id, bundle_hash, tx_hash, bid_id, signer, units, tip, status, submitted_timestamp, status_last_updated, signed_hash)
            VALUES('{auction_id}', NULL, NULL, '{bid_id}', '{signer}', {units}, {tip}, 'submitted', '{submitted_timestamp_iso}', '{now_iso}', '{sig}');
            ", 
                bid_id=Uuid::new_v4(),
                now_iso=now_iso,
                auction_id=bid.auction.id,
                signer=bid.payload.sender.to_lowercase(),
                units=bid.payload.get_values().nft_count,
                tip=bid.payload.get_values().tip_per_nft,
                submitted_timestamp_iso=bid.received_time.to_rfc3339(),
                sig=bid.payload.signature,
        );

        match client.query(&query, &[]).await {
            Ok(_) => Ok(()),
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
