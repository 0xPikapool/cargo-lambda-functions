use lambda_http::http::StatusCode;
use lambda_http::{Body, Request};
use pikapool_api::core::request_handler;
use pikapool_api::dummy_data;
use serde_json::to_string;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_handler_no_body() {
        let response = request_handler(Request::default()).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Request body missing"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_body() {
        let request = Request::new(Body::from("invalid body"));
        let response = request_handler(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "expected value at line 1 column 1"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_signer_address() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::BadSignerAddress);
        let request = Request::new(Body::from(to_string(&bid_request).unwrap()));
        let response = request_handler(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Invalid signer address"),
            _ => panic!("Malformed response"),
        }
    }
}
