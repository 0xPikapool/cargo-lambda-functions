use crate::auction::Auction;
use crate::bid_request::BidRequest;
use crate::database::{Database, RedisDatabase};
use crate::signature_validation::verify_signature;
use eip_712::hash_structured_data;
use ethers::types::Address;
use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Error, Request, Response};
use lazy_static::lazy_static;
use log::{info, warn};
use serde_json::from_str;
use std::env;
use std::str::FromStr;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use validator::Validate;

// Store the Database in a Mutex so we can reuse connections between
// lambda invocations
lazy_static! {
    static ref REDIS_DATABASE: Mutex<RedisDatabase> = {
        let redis_url = env::var("REDIS_URL").unwrap();
        let client = redis::Client::open(redis_url).unwrap();
        let db = RedisDatabase {
            client,
            connection: None,
        };
        Mutex::new(db)
    };
}

fn get_redis_database() -> &'static Mutex<RedisDatabase> {
    &REDIS_DATABASE
}

pub async fn request_handler(event: Request) -> Result<Response<Body>, Error> {
    match event.method() {
        &Method::PUT => {
            let db = get_redis_database();
            put_request_handler(event, db).await
        }
        &Method::OPTIONS => build_response(StatusCode::OK, "OK"),
        _ => build_response(StatusCode::NOT_IMPLEMENTED, "Method not implemented"),
    }
}

pub async fn put_request_handler(
    event: Request,
    db: &Mutex<impl Database>,
) -> Result<Response<Body>, Error> {
    // Deserialize the request body into a `BidPayload` struct
    info!("Deserializing request body");
    let bid_payload = match event.body() {
        Body::Text(body) => from_str::<BidRequest>(&body),
        _ => return build_response(StatusCode::BAD_REQUEST, "Request body missing"),
    };
    // Unwrap the EIP712 struct
    info!("Unwrapping EIP712 struct");
    let bid_payload = match bid_payload {
        Ok(payload) => payload,
        Err(e) => return build_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };
    // Validate the EIP712 msg is a valid Bid
    info!("Validating EIP712 msg");
    match bid_payload.validate() {
        Err(_) => {
            return build_response(
                StatusCode::BAD_REQUEST,
                "typed_data is not a valid Pikapool Bid",
            )
        }
        _ => (),
    };
    // Verify signer address
    info!("Verifying signer address");
    let signer_address = match Address::from_str(&bid_payload.sender) {
        Ok(address) => address,
        Err(_) => return build_response(StatusCode::BAD_REQUEST, "Invalid signer address"),
    };
    info!("Signer address: {}", signer_address);
    // Verify auction contract address is a valid Address
    info!("Verifying auction contract address");
    let auction_contract_address =
        match Address::from_str(&bid_payload.get_values().auction_contract) {
            Ok(address) => address,
            Err(_) => {
                return build_response(StatusCode::BAD_REQUEST, "Invalid auction contract address")
            }
        };
    info!("Auction contract address: {}", auction_contract_address);
    // Verify the signature
    info!("Verifying signature");
    let typed_data_hash_bytes: [u8; 32] = hash_structured_data(bid_payload.typed_data.clone())
        .unwrap()
        .into();
    match verify_signature(
        signer_address,
        typed_data_hash_bytes,
        &bid_payload.signature,
    ) {
        Ok(signature) => signature,
        Err(e) => return build_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    // Passed in-memory validation, now connect to DB
    let mut db = db.lock().unwrap();
    if !db.is_connected() {
        info!("Connecting to database");
        db.connect();
    } else {
        info!("Reusing database connection");
    }

    info!("Sleeping for 3 seconds...");
    sleep(Duration::from_secs(3));

    // Check auction is valid
    info!("Checking auction is valid");
    let auction: Auction = match db
        .get_auction(
            &bid_payload.typed_data.domain.chain_id.to_string(),
            &auction_contract_address,
        )
        .unwrap()
    {
        Some(option) => option,
        None => return build_response(StatusCode::BAD_REQUEST, "Specified auction does not exist"),
    };
    // Check user specified settlement contract matches actual settlement contract
    info!("Checking settlement contract matches");
    let settlement_contract_bytes: [u8; 20] =
        bid_payload.typed_data.domain.verifying_contract.into();
    let settlement_contract = Address::from_slice(&settlement_contract_bytes);
    if auction.settlement_contract != settlement_contract {
        return build_response(
            StatusCode::BAD_REQUEST,
            "Specified settlement contract does not match auction settlement contract",
        );
    }
    // Check user specified base_price matches actual base_price
    info!("Checking base_price matches");
    if auction.base_price != bid_payload.get_values().base_price_per_nft {
        return build_response(
            StatusCode::BAD_REQUEST,
            "Specified base_price does not match auction base_price",
        );
    }
    let cur_synced_block = db
        .get_synced_block(
            &bid_payload.typed_data.domain.chain_id.to_string(),
            &settlement_contract,
        )
        .unwrap();
    // Check that the auction has started
    info!("Checking auction has started");
    if cur_synced_block < auction.start_block {
        return build_response(StatusCode::BAD_REQUEST, "Auction has not started");
    }
    // Check that the auction has not ended
    info!("Checking auction has not ended");
    if cur_synced_block > auction.end_block {
        return build_response(StatusCode::BAD_REQUEST, "Auction has ended");
    }

    // Check user approval and balance
    info!("Getting user approval and balance");
    let (signer_approve_amt, signer_bal) = match db
        .get_signer_approve_and_bal_amts(
            &bid_payload.typed_data.domain.chain_id.to_string(),
            &settlement_contract,
            &signer_address,
        )
        .unwrap()
    {
        Some(option) => option,
        None => {
            return build_response(
                StatusCode::FORBIDDEN,
                "Signer has not approved the settlement contract",
            )
        }
    };
    // Verify user approval
    info!("Verifying user approval");
    let bid_cost = bid_payload.get_bid_cost();
    if signer_approve_amt < bid_cost {
        return build_response(
            StatusCode::FORBIDDEN,
            &"Signer approval amount is insufficient",
        );
    }
    // Verify user balance
    info!("Verifying user balance");
    if signer_bal < bid_cost {
        return build_response(
            StatusCode::FORBIDDEN,
            &"Signer token balance is insufficient",
        );
    }

    // PUSH TO SQS
    info!("OK!");
    build_response(StatusCode::OK, "OK")
}

fn build_response(status: StatusCode, message: &str) -> Result<Response<Body>, Error> {
    match status {
        StatusCode::OK => (),
        _ => warn!("{}: {}", status, message),
    };
    Ok(Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "PUT,OPTION")
        .header("Access-Control-Allow-Headers", "content-type")
        .status(status)
        .body(Body::from(message))
        .unwrap())
}
