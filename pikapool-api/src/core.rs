use crate::auction::Auction;
use crate::bid::Bid;
use crate::bid_payload::BidPayload;
use crate::cache::{Cache, RedisCache};
use crate::database::{Database, RdsProvider};
use crate::signature_validation::verify_signature;
use crate::utils::{lock_connectable_mutex_safely, Connectable};
use eip_712::hash_structured_data;
use ethers::types::Address;
use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Error, Request, Response};
use lazy_static::lazy_static;
use serde_json::from_str;
use std::str::FromStr;
use tokio::sync::Mutex;
use validator::Validate;

// Store the Cache in a Mutex so we can reuse connections between
// lambda invocations
lazy_static! {
    static ref REDIS_DATABASE: Mutex<RedisCache> = {
        let cache = RedisCache { connection: None };
        Mutex::new(cache)
    };
    static ref RDS_PROVIDER: Mutex<RdsProvider> = {
        let database = RdsProvider { client: None };
        Mutex::new(database)
    };
}

pub async fn request_handler(event: Request) -> Result<Response<Body>, Error> {
    let cache_mutex = &REDIS_DATABASE;
    let db = &RDS_PROVIDER;
    match event.method() {
        &Method::PUT => put_request_handler(event, cache_mutex, db).await,
        &Method::OPTIONS => build_response(StatusCode::OK, "OK"),
        _ => build_response(StatusCode::NOT_IMPLEMENTED, "Method not implemented"),
    }
}

pub async fn put_request_handler(
    event: Request,
    cache_mutex: &Mutex<impl Cache>,
    db_mutex: &Mutex<impl Database>,
) -> Result<Response<Body>, Error> {
    let bid = match parse_and_validate_event(event, cache_mutex).await {
        Ok(bid_payload) => bid_payload,
        Err(e) => return e,
    };

    println!("Connecting to DB");
    let mut db = match lock_connectable_mutex_safely(db_mutex).await {
        Ok(db) => db,
        Err(e) => return build_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    println!("Sending to db...");
    match db.insert_bid(&bid).await {
        Ok(hash) => {
            println!("Done! Returning 200.");
            build_response(StatusCode::OK, &hash)
        }
        Err(e) => {
            eprintln!("Error sending to db: {}", e);
            build_response(StatusCode::INTERNAL_SERVER_ERROR, &e)
        }
    }
}

pub async fn parse_and_validate_event(
    event: Request,
    cache_mutex: &Mutex<impl Cache + Connectable>,
) -> Result<Bid, Result<Response<Body>, Error>> {
    let received_time = chrono::Utc::now();

    // Deserialize the request body into a `BidPayload` struct
    println!("Deserializing request body");
    let bid_payload = match event.body() {
        Body::Text(body) => from_str::<BidPayload>(&body),
        _ => {
            return Err(build_response(
                StatusCode::BAD_REQUEST,
                "Request body missing",
            ))
        }
    };
    // Unwrap the EIP712 struct
    println!("Unwrapping EIP712 struct");
    let bid_payload = match bid_payload {
        Ok(payload) => payload,
        Err(e) => return Err(build_response(StatusCode::BAD_REQUEST, &e.to_string())),
    };
    let parsed_bid_values = match bid_payload.parse_values() {
        Ok(parsed_bid_values) => parsed_bid_values,
        Err(e) => return Err(build_response(StatusCode::BAD_REQUEST, &e.to_string())),
    };
    // Validate the EIP712 msg is a valid Bid
    println!("Validating EIP712 msg");
    match bid_payload.validate() {
        Err(_) => {
            return Err(build_response(
                StatusCode::BAD_REQUEST,
                "typed_data is not a valid Pikapool Bid",
            ))
        }
        _ => (),
    };
    // Verify signer address
    println!("Verifying signer address");
    let signer_address = match Address::from_str(&bid_payload.sender) {
        Ok(address) => address,
        Err(_) => {
            return Err(build_response(
                StatusCode::BAD_REQUEST,
                "Invalid signer address",
            ))
        }
    };
    println!("Signer address: {}", signer_address);
    // Verify auction contract address is a valid Address
    println!("Verifying auction contract address");
    let auction_contract_address = match Address::from_str(&parsed_bid_values.auction_address) {
        Ok(address) => address,
        Err(_) => {
            return Err(build_response(
                StatusCode::BAD_REQUEST,
                "Invalid auction contract address",
            ))
        }
    };
    println!("Auction contract address: {}", auction_contract_address);
    // Verify the signature
    println!("Verifying signature");
    let typed_data_hash_bytes: [u8; 32] = match hash_structured_data(bid_payload.typed_data.clone())
    {
        Ok(hash) => hash.into(),
        Err(e) => return Err(build_response(StatusCode::BAD_REQUEST, &e.to_string())),
    };
    match verify_signature(
        signer_address,
        typed_data_hash_bytes,
        &bid_payload.signature,
    ) {
        Ok(signature) => signature,
        Err(e) => return Err(build_response(StatusCode::BAD_REQUEST, &e.to_string())),
    };

    // Passed in-memory validation, now connect to DB
    println!("Connecting to Cache");
    let mut cache = match lock_connectable_mutex_safely(cache_mutex).await {
        Ok(cache) => cache,
        Err(e) => {
            return Err(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };

    // Check auction is valid
    println!("Checking auction is valid");
    let auction: Auction = match cache.get_auction(
        &bid_payload.typed_data.domain.chain_id.to_string(),
        &auction_contract_address,
        &parsed_bid_values.auction_name,
    ) {
        Ok(a) => match a {
            Some(option) => option,
            None => {
                return Err(build_response(
                    StatusCode::BAD_REQUEST,
                    "Specified auction does not exist",
                ))
            }
        },
        Err(e) => {
            return Err(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };
    // Check user specified settlement contract matches actual settlement contract
    println!("Checking settlement contract matches");
    let settlement_contract_bytes: [u8; 20] =
        bid_payload.typed_data.domain.verifying_contract.into();
    let settlement_contract = Address::from_slice(&settlement_contract_bytes);
    if auction.settlement_contract != settlement_contract {
        return Err(build_response(
            StatusCode::BAD_REQUEST,
            "Specified settlement contract does not match auction settlement contract",
        ));
    }
    // Check user specified base_price matches actual base_price
    println!("Checking base_price matches");
    if auction.base_price != parsed_bid_values.base_price {
        return Err(build_response(
            StatusCode::BAD_REQUEST,
            "Specified base_price does not match auction base_price",
        ));
    }
    // Check user specified base_price matches actual base_price
    println!("Getting current block");
    let cur_synced_block = match cache.get_synced_block(
        &bid_payload.typed_data.domain.chain_id.to_string(),
        &settlement_contract,
    ) {
        Ok(block) => block,
        Err(e) => {
            return Err(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };
    // Check that the auction has started
    println!("Checking auction has started");
    if cur_synced_block < auction.start_block {
        return Err(build_response(
            StatusCode::BAD_REQUEST,
            "Auction has not started",
        ));
    }
    // Check that the auction has not ended
    println!("Checking auction has not ended");
    if cur_synced_block > auction.end_block {
        return Err(build_response(StatusCode::BAD_REQUEST, "Auction has ended"));
    }

    // Check user approval and balance
    println!("Getting user approval and balance");
    let (signer_approve_amt, signer_bal) = match cache.get_signer_approve_and_bal_amts(
        &bid_payload.typed_data.domain.chain_id.to_string(),
        &settlement_contract,
        &signer_address,
    ) {
        Ok(res) => match res {
            Some(option) => option,
            None => {
                return Err(build_response(
                    StatusCode::FORBIDDEN,
                    "Signer has not approved the settlement contract",
                ))
            }
        },
        Err(e) => {
            return Err(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };
    // Verify user approval
    println!("Verifying user approval");
    let bid_cost = parsed_bid_values.get_bid_cost();
    if signer_approve_amt < bid_cost {
        return Err(build_response(
            StatusCode::FORBIDDEN,
            &"Signer approval amount is insufficient",
        ));
    }
    // Verify user balance
    println!("Verifying user balance");
    if signer_bal < bid_cost {
        return Err(build_response(
            StatusCode::FORBIDDEN,
            &"Signer token balance is insufficient",
        ));
    };

    println!("Valid!");
    Ok(Bid::new(
        bid_payload,
        parsed_bid_values,
        received_time,
        auction,
    ))
}

fn build_response(status: StatusCode, message: &str) -> Result<Response<Body>, Error> {
    match status {
        StatusCode::OK => (),
        _ => eprintln!("{}: {}", status, message),
    };
    let res = match Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "PUT,OPTION")
        .header("Access-Control-Allow-Headers", "content-type")
        .status(status)
        .body(Body::from(message))
    {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to build response: {}", e);
            return Err(Box::new(e));
        }
    };
    Ok(res)
}
