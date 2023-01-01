use crate::bid_request::BidRequest;
use crate::database::{Database, RedisDatabase};
use crate::signature_validation::verify_signature;
use eip_712::hash_structured_data;
use ethers::types::Address;
use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Error, Request, Response};
use serde_json::from_str;
use std::env;
use std::str::FromStr;
use validator::Validate;

pub async fn request_handler(event: Request) -> Result<Response<Body>, Error> {
    match event.method() {
        &Method::PUT => {
            let redis_url = env::var("REDIS_URL").unwrap();
            let client = redis::Client::open(redis_url).unwrap();
            let mut db = RedisDatabase {
                client,
                connection: None,
            };
            put_request_handler(event, &mut db).await
        }
        &Method::OPTIONS => build_response(StatusCode::OK, "OK"),
        _ => build_response(StatusCode::NOT_IMPLEMENTED, "Method not implemented"),
    }
}

pub async fn put_request_handler(
    event: Request,
    db: &mut impl Database,
) -> Result<Response<Body>, Error> {
    // Deserialize the request body into a `BidPayload` struct
    let bid_payload = match event.body() {
        Body::Text(body) => from_str::<BidRequest>(&body),
        _ => return build_response(StatusCode::BAD_REQUEST, "Request body missing"),
    };
    // Unwrap the EIP712 struct
    let bid_payload = match bid_payload {
        Ok(payload) => payload,
        Err(e) => return build_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };
    // Validate the EIP712 msg is a valid Bid
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
    let signer = match Address::from_str(&bid_payload.sender) {
        Ok(address) => address,
        Err(_) => return build_response(StatusCode::BAD_REQUEST, "Invalid signer address"),
    };
    // Verify the signature
    let typed_data_hash_bytes: [u8; 32] = hash_structured_data(bid_payload.typed_data.clone())
        .unwrap()
        .into();
    match verify_signature(signer, typed_data_hash_bytes, &bid_payload.signature) {
        Ok(signature) => signature,
        Err(e) => return build_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    // Passed in-memory validation, now connect to DB
    // TODO: Persist the database connection between calls
    db.connect();

    // Check user approval and balance
    let (signer_approve_amt, signer_bal) = match db
        .get_signer_approve_and_bal_amts(
            &bid_payload.typed_data.domain.chain_id.to_string(),
            &bid_payload.typed_data.domain.verifying_contract.to_string(),
            &bid_payload.sender,
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
    let bid_cost = bid_payload.get_bid_cost();
    if signer_approve_amt < bid_cost {
        return build_response(
            StatusCode::FORBIDDEN,
            &"Signer approval amount is insufficient",
        );
    }
    // Verify user balance
    if signer_bal < bid_cost {
        return build_response(
            StatusCode::FORBIDDEN,
            &"Signer token balance is insufficient",
        );
    }

    build_response(
        StatusCode::OK,
        serde_json::to_string(&bid_payload).unwrap().as_str(),
    )
}

fn build_response(status: StatusCode, message: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "PUT,OPTION")
        .header("Access-Control-Allow-Headers", "content-type")
        .status(status)
        .body(Body::from(message))
        .unwrap())
}
