use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Request};
use pikapool_api::core::put_request_handler;
use pikapool_api::database::MockDatabase;
use pikapool_api::dummy_data;
use serde_json::to_string;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_handler_no_body() {
        let mut mock_database = MockDatabase::new();
        let mut r = Request::default();
        *r.method_mut() = Method::PUT;
        let response = put_request_handler(r, &mut mock_database).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Request body missing"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_eip712() {
        let mut r = Request::new(Body::from("invalid body"));
        *r.method_mut() = Method::PUT;
        let mut mock_database = MockDatabase::new();
        let response = put_request_handler(r, &mut mock_database).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "expected value at line 1 column 1"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_bid() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::InvalidBid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = MockDatabase::new();
        let response = put_request_handler(r, &mut mock_database).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "typed_data is not a valid Pikapool Bid"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_signer_address() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::BadSignerAddress);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = MockDatabase::new();
        let response = put_request_handler(r, &mut mock_database).await.unwrap();

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
        let mut mock_database = MockDatabase::new();
        let response = put_request_handler(r, &mut mock_database).await.unwrap();

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
        let mut mock_database = MockDatabase::new();
        let response = put_request_handler(r, &mut mock_database).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Signature does not match signer"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_no_signer_approval() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = MockDatabase::new();

        mock_database.expect_connect().returning(|| ());
        mock_database
            .expect_get_signer_approve_and_bal_amts()
            .returning(|_, _, _| Ok(None));

        let response = put_request_handler(r, &mut mock_database).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Signer has not approved the settlement contract"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_approval_too_low() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = MockDatabase::new();

        mock_database.expect_connect().returning(|| ());
        mock_database
            .expect_get_signer_approve_and_bal_amts()
            .returning(|_, _, _| Ok(Some((0.5, 200 as f64))));

        let response = put_request_handler(r, &mut mock_database).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Signer approval amount is insufficient"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_balance_too_low() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = MockDatabase::new();

        mock_database.expect_connect().returning(|| ());
        mock_database
            .expect_get_signer_approve_and_bal_amts()
            .returning(|_, _, _| Ok(Some((200 as f64, 0.1))));

        let response = put_request_handler(r, &mut mock_database).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Signer token balance is insufficient"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_sig_happy_path() {
        let bid_request = dummy_data::new_bid_request(dummy_data::Option::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = MockDatabase::new();

        mock_database.expect_connect().returning(|| ());
        mock_database
            .expect_get_signer_approve_and_bal_amts()
            .returning(|_, _, _| Ok(Some((200.1, 200.1))));

        let response = put_request_handler(r, &mut mock_database).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
