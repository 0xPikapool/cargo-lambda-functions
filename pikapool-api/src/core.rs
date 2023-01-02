use crate::auction::Auction;
use crate::bid_request::BidRequest;
use crate::cache::{Cache, RedisCache};
use crate::lock_result_ext::LockResultExt;
use crate::signature_validation::verify_signature;
use crate::sink::{Sink, SqsProvider};
use eip_712::hash_structured_data;
use ethers::types::Address;
use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Error, Request, Response};
use lazy_static::lazy_static;
use serde_json::{from_str, json};
use std::str::FromStr;
use std::sync::Mutex;
use validator::Validate;

// Store the Cache in a Mutex so we can reuse connections between
// lambda invocations
lazy_static! {
    static ref REDIS_DATABASE: Mutex<RedisCache> = {
        let cache = RedisCache { connection: None };
        Mutex::new(cache)
    };
}

pub async fn request_handler(event: Request) -> Result<Response<Body>, Error> {
    let cache_mutex = &REDIS_DATABASE;
    let mut sink = SqsProvider {};
    match event.method() {
        &Method::PUT => put_request_handler(event, cache_mutex, &mut sink).await,
        &Method::OPTIONS => build_response(StatusCode::OK, "OK"),
        _ => build_response(StatusCode::NOT_IMPLEMENTED, "Method not implemented"),
    }
}

pub async fn put_request_handler(
    event: Request,
    cache_mutex: &Mutex<impl Cache>,
    sink: &mut impl Sink,
) -> Result<Response<Body>, Error> {
    let bid_request = match parse_and_validate_event(event, cache_mutex) {
        Ok(bid_request) => bid_request,
        Err(e) => return e,
    };

    println!("Sending to sink...");
    match sink.send(&json!(bid_request).to_string()).await {
        Ok(_) => {
            println!("Done! Returning 200.");
            build_response(StatusCode::OK, "OK")
        }
        Err(e) => {
            eprintln!("Error sending to sink: {}", e);
            build_response(StatusCode::INTERNAL_SERVER_ERROR, &e)
        }
    }
}

pub fn parse_and_validate_event(
    event: Request,
    cache_mutex: &Mutex<impl Cache>,
) -> Result<BidRequest, Result<Response<Body>, Error>> {
    // Deserialize the request body into a `BidPayload` struct
    println!("Deserializing request body");
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
    println!("Unwrapping EIP712 struct");
    let bid_payload = match bid_payload {
        Ok(payload) => payload,
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
    println!("Auction contract address: {}", auction_contract_address);
    // Verify the signature
    println!("Verifying signature");
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
    let mut cache = cache_mutex.lock().ignore_poison();
    if cache_mutex.is_poisoned() {
        eprintln!(
            "Cache connection is poisoned, this should never happen. Forcing a reconnection."
        );
    }
    if cache_mutex.is_poisoned() || !cache.is_connected() {
        println!("Establishing new connection to Redis");
        match cache.connect() {
            Ok(_) => (),
            Err(e) => {
                return Err(build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &e.to_string(),
                ))
            }
        };
    } else {
        println!("Reusing database connection ⚡");
        match cache.ping() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Ping failed: {}. Attempting to reconnect...", e.to_string());
                match cache.connect() {
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
    println!("Checking auction is valid");
    let auction: Auction = match cache.get_auction(
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
    if auction.base_price != bid_payload.get_values().base_price_per_nft {
        return Err(build_response(
            StatusCode::BAD_REQUEST,
            "Specified base_price does not match auction base_price",
        ));
    }
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
    let bid_cost = bid_payload.get_bid_cost();
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
    Ok(bid_payload)
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
