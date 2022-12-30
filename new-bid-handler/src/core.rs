use crate::signature_validation::verify_signature;
use eip_712::{hash_structured_data, EIP712};
use ethers::types::Address;
use lambda_http::http::StatusCode;
use lambda_http::{Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub struct BidRequest {
    pub typed_data: EIP712,
    pub sender: String,
    pub signature: String,
}

// TODO: Reuse redis connections between invocations
// #[macro_use]
// extern crate lazy_static;
// lazy_static! {
//     #[derive(Debug)]
//     static ref REDIS_CONNECTION: redis::Connection =
//         redis::Client::open(env::var("REDIS_URL").unwrap()).unwrap().get_connection().unwrap();
// }

pub async fn request_handler(event: Request) -> Result<Response<Body>, Error> {
    // Deserialize the request body into a `BidPayload` struct, return 400 if there's no body
    let bid_payload = match event.body() {
        Body::Text(body) => from_str::<BidRequest>(&body),
        _ => return build_response(StatusCode::BAD_REQUEST, "Request body missing"),
    };
    // Unwrap the bid payload, return 400 if the body is invalid
    let bid_payload = match bid_payload {
        Ok(payload) => payload,
        Err(e) => return build_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };
    // Verify signer address
    let signer = match Address::from_str(&bid_payload.sender) {
        Ok(address) => address,
        Err(_) => return build_response(StatusCode::BAD_REQUEST, "Invalid signer address"),
    };
    // Verify the signature, return 400 if the signature is invalid
    let typed_data_hash_bytes: [u8; 32] = hash_structured_data(bid_payload.typed_data.clone())
        .unwrap()
        .into();
    match verify_signature(signer, typed_data_hash_bytes, &bid_payload.signature) {
        Ok(signature) => signature,
        Err(e) => return build_response(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    // // Spin up Redis connection
    // let redis_url = env::var("REDIS_URL").unwrap();
    // let redis_client = redis::Client::open(redis_url).unwrap();
    // let mut redis_connection = redis_client.get_connection().unwrap();

    // let synced_block = redis::cmd("GET")
    //     .arg("synced_block")
    //     .query::<String>(&mut redis_connection)
    //     .unwrap();
    // println!("Synced block: {}", synced_block);

    build_response(
        StatusCode::OK,
        serde_json::to_string(&bid_payload).unwrap().as_str(),
    )
}

fn build_response(status: StatusCode, message: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .body(Body::from(message))
        .unwrap())
}
