use crate::bid_request::BidRequest;
use crate::signature_validation::verify_signature;
use eip_712::hash_structured_data;
use ethers::types::Address;
use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Error, Request, Response};
use serde_json::from_str;
use std::env;
use std::format;
use std::str::FromStr;
use validator::Validate;

pub async fn request_handler(event: Request) -> Result<Response<Body>, Error> {
    match event.method() {
        &Method::PUT => put_request_handler(event).await,
        &Method::OPTIONS => build_response(StatusCode::OK, "OK"),
        _ => build_response(StatusCode::NOT_IMPLEMENTED, "Method not implemented"),
    }
}

pub async fn put_request_handler(event: Request) -> Result<Response<Body>, Error> {
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

    // Spin up Redis connection
    let redis_url = env::var("REDIS_URL").unwrap();
    println!("Connecting to Redis at {}", redis_url);
    let redis_client = redis::Client::open(redis_url).unwrap();
    let mut redis_connection = redis_client.get_connection().unwrap();

    let synced_block: Option<String> = redis::cmd("GET")
        .arg("synced_block")
        .query(&mut redis_connection)
        .unwrap();
    println!("Synced block: {:?}", synced_block);

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
