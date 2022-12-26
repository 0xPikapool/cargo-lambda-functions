use lambda_http::{run, service_fn, Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct BidRequest {
    auction: String,
    from: String,
    amount: u64,
    tip: u64,
    signed_msg: String,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Deserialize the request body into a `BidPayload` struct, return 400 if there's no body
    let bid_payload = match event.body() {
        Body::Text(body) => serde_json::from_str::<BidRequest>(&body),
        _ => return build_response(400, "No body"),
    };
    // Unwrap the bid payload, return 400 if the body is invalid
    let bid_payload = match bid_payload {
        Ok(payload) => payload,
        Err(e) => return build_response(400, &e.to_string()),
    };

    build_response(200, serde_json::to_string(&bid_payload).unwrap().as_str())
}

fn build_response(status: u16, message: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .body(Body::from(message))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
