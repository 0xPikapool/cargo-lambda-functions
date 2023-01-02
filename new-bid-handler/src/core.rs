use crate::auction::Auction;
use crate::bid_request::BidRequest;
use crate::database::{Database, RedisDatabase};
use crate::lock_result_ext::LockResultExt;
use crate::signature_validation::verify_signature;
use crate::sink::{Sink, SqsProvider};
use eip_712::hash_structured_data;
use ethers::types::Address;
use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Error, Request, Response};
use lazy_static::lazy_static;
use log::{error, info, warn};
use serde_json::{from_str, json};
use std::str::FromStr;
use std::sync::Mutex;
use validator::Validate;

// Store the Database in a Mutex so we can reuse connections between
// lambda invocations
lazy_static! {
    static ref REDIS_DATABASE: Mutex<RedisDatabase> = {
        let db = RedisDatabase { connection: None };
        Mutex::new(db)
    };
}

pub async fn request_handler(event: Request) -> Result<Response<Body>, Error> {
    let db_mutex = &REDIS_DATABASE;
    let mut sink = SqsProvider {};
    match event.method() {
        &Method::PUT => put_request_handler(event, db_mutex, &mut sink).await,
        &Method::OPTIONS => build_response(StatusCode::OK, "OK"),
        _ => build_response(StatusCode::NOT_IMPLEMENTED, "Method not implemented"),
    }
}

pub async fn put_request_handler(
    event: Request,
    db_mutex: &Mutex<impl Database>,
    sink: &mut impl Sink,
) -> Result<Response<Body>, Error> {
    let bid_request = match parse_and_validate_event(event, db_mutex) {
        Ok(bid_request) => bid_request,
        Err(e) => return e,
    };

    match sink.send(&json!(bid_request).to_string()).await {
        Ok(_) => build_response(StatusCode::OK, "OK"),
        Err(e) => build_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}

pub fn parse_and_validate_event(
    event: Request,
    db_mutex: &Mutex<impl Database>,
) -> Result<BidRequest, Result<Response<Body>, Error>> {
    // Deserialize the request body into a `BidPayload` struct
    info!("Deserializing request body");
    let bid_payload = match event.body() {
        Body::Text(body) => from_str::<BidRequest>(&body),
        _ => {
            return Err(build_response(
                StatusCode::BAD_REQUEST,
                "Request body missing",
            ))
        }
    };
    // Unwrap the EIP712 struct
    info!("Unwrapping EIP712 struct");
    let bid_payload = match bid_payload {
        Ok(payload) => payload,
        Err(e) => return Err(build_response(StatusCode::BAD_REQUEST, &e.to_string())),
    };
    // Validate the EIP712 msg is a valid Bid
    info!("Validating EIP712 msg");
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
    info!("Verifying signer address");
    let signer_address = match Address::from_str(&bid_payload.sender) {
        Ok(address) => address,
        Err(_) => {
            return Err(build_response(
                StatusCode::BAD_REQUEST,
                "Invalid signer address",
            ))
        }
    };
    info!("Signer address: {}", signer_address);
    // Verify auction contract address is a valid Address
    info!("Verifying auction contract address");
    let auction_contract_address =
        match Address::from_str(&bid_payload.get_values().auction_contract) {
            Ok(address) => address,
            Err(_) => {
                return Err(build_response(
                    StatusCode::BAD_REQUEST,
                    "Invalid auction contract address",
                ))
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
        Err(e) => return Err(build_response(StatusCode::BAD_REQUEST, &e.to_string())),
    };

    // Passed in-memory validation, now connect to DB
    let mut db = db_mutex.lock().ignore_poison();
    if db_mutex.is_poisoned() {
        warn!("Database connection is poisoned, this should never happen. Forcing a reconnection.");
    }
    if db_mutex.is_poisoned() || !db.is_connected() {
        info!("Establishing new connection to Redis");
        match db.connect() {
            Ok(_) => (),
            Err(e) => {
                return Err(build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &e.to_string(),
                ))
            }
        };
    } else {
        info!("Reusing database connection ⚡");
        match db.ping() {
            Ok(_) => (),
            Err(e) => {
                error!("Ping failed: {}. Attempting to reconnect...", e.to_string());
                match db.connect() {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(build_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            &e.to_string(),
                        ))
                    }
                };
            }
        }
    }

    // Check auction is valid
    info!("Checking auction is valid");
    let auction: Auction = match db.get_auction(
        &bid_payload.typed_data.domain.chain_id.to_string(),
        &auction_contract_address,
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
    info!("Checking settlement contract matches");
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
    info!("Checking base_price matches");
    if auction.base_price != bid_payload.get_values().base_price_per_nft {
        return Err(build_response(
            StatusCode::BAD_REQUEST,
            "Specified base_price does not match auction base_price",
        ));
    }
    let cur_synced_block = match db.get_synced_block(
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
    info!("Checking auction has started");
    if cur_synced_block < auction.start_block {
        return Err(build_response(
            StatusCode::BAD_REQUEST,
            "Auction has not started",
        ));
    }
    // Check that the auction has not ended
    info!("Checking auction has not ended");
    if cur_synced_block > auction.end_block {
        return Err(build_response(StatusCode::BAD_REQUEST, "Auction has ended"));
    }

    // Check user approval and balance
    info!("Getting user approval and balance");
    let (signer_approve_amt, signer_bal) = match db.get_signer_approve_and_bal_amts(
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
    info!("Verifying user approval");
    let bid_cost = bid_payload.get_bid_cost();
    if signer_approve_amt < bid_cost {
        return Err(build_response(
            StatusCode::FORBIDDEN,
            &"Signer approval amount is insufficient",
        ));
    }
    // Verify user balance
    info!("Verifying user balance");
    if signer_bal < bid_cost {
        return Err(build_response(
            StatusCode::FORBIDDEN,
            &"Signer token balance is insufficient",
        ));
    };

    info!("Valid!");
    Ok(bid_payload)
}

fn build_response(status: StatusCode, message: &str) -> Result<Response<Body>, Error> {
    match status {
        StatusCode::OK => (),
        _ => warn!("{}: {}", status, message),
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
            error!("Failed to build response: {}", e);
            return Err(Box::new(e));
        }
    };
    Ok(res)
}
