use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Request};
use pikapool_api::core::request_handler;
use pikapool_api::dummy_data;
use serde_json::to_string;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_handler_no_body() {
        let mut r = Request::default();
        *r.method_mut() = Method::PUT;
        let response = request_handler(r).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Request body missing"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_body() {
        let mut r = Request::new(Body::from("invalid body"));
        *r.method_mut() = Method::PUT;
        let response = request_handler(r).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "expected value at line 1 column 1"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_signer_address() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::BadSignerAddress);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let response = request_handler(r).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Invalid signer address"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_sig() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::InvalidSignature);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let response = request_handler(r).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Invalid signature"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_sig_doesnt_match_signer() {
        let bid_request =
            dummy_data::new_bid_request(dummy_data::Option::SignatureDoesNotMatchSigner);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let response = request_handler(r).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Signature does not match signer"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_sig_matches_signer() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let response = request_handler(r).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
