use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Request};
use pikapool_api::core::put_request_handler;
use pikapool_api::database::MockDatabase;
use pikapool_api::dummy_data;
use pikapool_api::sink::MockSink;
use serde_json::to_string;
use std::sync::Mutex;

fn with_lock<T, F, R>(mutex: &Mutex<T>, f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    let mut guard = mutex.lock().unwrap();
    f(&mut *guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_handler_no_body() {
        let mut mock_database = Mutex::new(MockDatabase::new());
        let mut mock_sink = MockSink::new();
        let mut r = Request::default();
        *r.method_mut() = Method::PUT;
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

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
        let mut mock_database = Mutex::new(MockDatabase::new());
        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "expected value at line 1 column 1"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_bid() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::InvalidBid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());
        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "typed_data is not a valid Pikapool Bid"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_signer_address() {
        let bid_request =
            dummy_data::new_bid_request(dummy_data::BidRequestOption::BadSignerAddress);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());
        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Invalid signer address"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_auction_address() {
        let bid_request =
            dummy_data::new_bid_request(dummy_data::BidRequestOption::InvalidAuctionAddress);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());
        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Invalid auction contract address"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_sig() {
        let bid_request =
            dummy_data::new_bid_request(dummy_data::BidRequestOption::InvalidSignature);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());
        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Invalid signature"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_sig_doesnt_match_signer() {
        let bid_request =
            dummy_data::new_bid_request(dummy_data::BidRequestOption::SignatureDoesNotMatchSigner);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());
        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Signature does not match signer"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_no_auction() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());

        with_lock(&mut mock_database, |db| {
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_is_connected().returning(|| true);
            db.expect_get_auction().returning(|_, _| Ok(None));
        });

        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Specified auction does not exist"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_settlement_contract_doesnt_match() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());

        with_lock(&mut mock_database, |db| {
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_is_connected().returning(|| true);
            db.expect_get_auction().returning(|_, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::InvalidSettlementAddress,
                )))
            });
        });

        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "Specified settlement contract does not match auction settlement contract"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_base_price_doesnt_match() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());

        with_lock(&mut mock_database, |db| {
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_is_connected().returning(|| true);
            db.expect_get_auction().returning(|_, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::InvalidBasePrice,
                )))
            });
        });

        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "Specified base_price does not match auction base_price"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_auction_not_started() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());

        with_lock(&mut mock_database, |db| {
            db.expect_is_connected().returning(|| true);
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| Ok(Some((200.1, 200.1))));
            db.expect_get_synced_block().returning(|_, _| Ok(99));
            db.expect_get_auction().returning(|_, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        });

        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Auction has not started"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_auction_has_ended() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());

        with_lock(&mut mock_database, |db| {
            db.expect_is_connected().returning(|| true);
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| Ok(Some((200.1, 200.1))));
            db.expect_get_synced_block().returning(|_, _| Ok(201));
            db.expect_get_auction().returning(|_, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        });

        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Auction has ended"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_no_signer_approval() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());

        with_lock(&mut mock_database, |db| {
            db.expect_is_connected().returning(|| true);
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| Ok(None));
            db.expect_get_synced_block().returning(|_, _| Ok(150));
            db.expect_get_auction().returning(|_, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        });

        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Signer has not approved the settlement contract"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_approval_too_low() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());

        with_lock(&mut mock_database, |db| {
            db.expect_is_connected().returning(|| true);
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| Ok(Some((0.5, 0.1))));
            db.expect_get_synced_block().returning(|_, _| Ok(150));
            db.expect_get_auction().returning(|_, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        });

        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Signer approval amount is insufficient"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_balance_too_low() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());

        with_lock(&mut mock_database, |db| {
            db.expect_is_connected().returning(|| true);
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| Ok(Some((200.1, 0.1))));
            db.expect_get_synced_block().returning(|_, _| Ok(150));
            db.expect_get_auction().returning(|_, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        });

        let mut mock_sink = MockSink::new();
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "Signer token balance is insufficient"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_sig_happy_path() {
        let bid_request = dummy_data::new_bid_request(dummy_data::BidRequestOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_request).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_database = Mutex::new(MockDatabase::new());

        with_lock(&mut mock_database, |db| {
            db.expect_is_connected().returning(|| true);
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| Ok(Some((200.1, 200.1))));
            db.expect_get_synced_block().returning(|_, _| Ok(150));
            db.expect_get_auction().returning(|_, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        });

        let mut mock_sink = MockSink::new();
        mock_sink.expect_send().returning(|_| Ok(()));
        let response = put_request_handler(r, &mut mock_database, &mut mock_sink)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
