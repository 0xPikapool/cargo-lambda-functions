use async_trait::async_trait;
use ethers::types::{Address, U256};
use lambda_http::http::{Method, StatusCode};
use lambda_http::{Body, Request};
use mockall::{mock, predicate::*};
use pikapool_api::auction::Auction;
use pikapool_api::bid::Bid;
use pikapool_api::cache::Cache as RealCache;
use pikapool_api::core::put_request_handler;
use pikapool_api::database::Database as RealDatabase;
use pikapool_api::dummy_data;
use pikapool_api::utils::Connectable;
use serde_json::to_string;
use tokio::sync::Mutex;

mock! {
    Database {}

    #[async_trait]
    impl Connectable for Database {
        async fn connect(&mut self) -> Result<(), String>;
        async fn ping(&mut self) -> Result<(), String>;
        async fn is_connected(&self) -> bool;
    }

    #[async_trait]
    impl RealDatabase for Database {
        async fn insert_bid(&mut self, bid: &Bid) -> Result<String, String>;
    }
}

mock! {
    Cache {}

    #[async_trait]
    impl Connectable for Cache {
        async fn connect(&mut self) -> Result<(), String>;
        async fn ping(&mut self) -> Result<(), String>;
        async fn is_connected(&self) -> bool;
    }

    #[async_trait]
    impl RealCache for Cache {
        fn get_signer_approve_and_bal_amts(
            &mut self,
            chain_id: &str,
            verifying_contract: &Address,
            signer: &Address,
        ) -> Result<Option<(U256, U256)>, String>;
        fn get_auction(
            &mut self,
            chain_id: &str,
            auction_contract: &Address,
            auction_name: &str,
        ) -> Result<Option<Auction>, String>;
        fn get_synced_block(
            &mut self,
            chain_id: &str,
            settlement_contract: &Address,
        ) -> Result<u64, String>;
    }
}

async fn with_lock<T, F, R>(mutex: &Mutex<T>, f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    let mut guard = mutex.lock().await;
    f(&mut *guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_handler_no_body() {
        let mock_cache = Mutex::new(MockCache::new());
        let mock_db = Mutex::new(MockDatabase::new());
        let mut r = Request::default();
        *r.method_mut() = Method::PUT;
        let response = put_request_handler(r, &mock_cache, &mock_db).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Request body missing\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_eip712() {
        let mut r = Request::new(Body::from("invalid body"));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());
        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"expected value at line 1 column 1\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_bid() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::InvalidBid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());
        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"typed_data is not a valid Pikapool Bid\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_signer_address() {
        let bid_payload =
            dummy_data::new_bid_payload(dummy_data::BidPayloadOption::BadSignerAddress);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());
        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Invalid signer address\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_auction_address() {
        let bid_payload =
            dummy_data::new_bid_payload(dummy_data::BidPayloadOption::InvalidAuctionAddress);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());
        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Invalid auction contract address\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_invalid_sig() {
        let bid_payload =
            dummy_data::new_bid_payload(dummy_data::BidPayloadOption::InvalidSignature);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());
        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Invalid signature\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_sig_doesnt_match_signer() {
        let bid_payload =
            dummy_data::new_bid_payload(dummy_data::BidPayloadOption::SignatureDoesNotMatchSigner);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());
        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Signature does not match signer\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_no_auction() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());

        with_lock(&mut mock_cache, |cache| {
            cache.expect_connect().returning(|| Ok(()));
            cache.expect_ping().returning(|| Ok(()));
            cache.expect_is_connected().returning(|| true);
            cache.expect_get_auction().returning(|_, _, _| Ok(None));
        })
        .await;

        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Specified auction does not exist\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_settlement_contract_doesnt_match() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());

        with_lock(&mut mock_cache, |cache| {
            cache.expect_connect().returning(|| Ok(()));
            cache.expect_ping().returning(|| Ok(()));
            cache.expect_is_connected().returning(|| true);
            cache.expect_get_auction().returning(|_, _, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::InvalidSettlementAddress,
                )))
            });
        })
        .await;

        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Specified settlement contract does not match auction settlement contract\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_base_price_doesnt_match() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());

        with_lock(&mut mock_cache, |cache| {
            cache.expect_connect().returning(|| Ok(()));
            cache.expect_ping().returning(|| Ok(()));
            cache.expect_is_connected().returning(|| true);
            cache.expect_get_auction().returning(|_, _, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::InvalidBasePrice,
                )))
            });
        })
        .await;

        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Specified base_price does not match auction base_price\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_auction_not_started() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());

        with_lock(&mut mock_cache, |cache| {
            cache.expect_is_connected().returning(|| true);
            cache.expect_connect().returning(|| Ok(()));
            cache.expect_ping().returning(|| Ok(()));
            cache
                .expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| {
                    Ok(Some((
                        200000000000000000000u128.into(),
                        200000000000000000000u128.into(),
                    )))
                });
            cache.expect_get_synced_block().returning(|_, _| Ok(99));
            cache.expect_get_auction().returning(|_, _, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        })
        .await;

        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Auction has not started\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_auction_has_ended() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());

        with_lock(&mut mock_cache, |cache| {
            cache.expect_is_connected().returning(|| true);
            cache.expect_connect().returning(|| Ok(()));
            cache.expect_ping().returning(|| Ok(()));
            cache
                .expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| {
                    Ok(Some((
                        200000000000000000000u128.into(),
                        200000000000000000000u128.into(),
                    )))
                });
            cache.expect_get_synced_block().returning(|_, _| Ok(201));
            cache.expect_get_auction().returning(|_, _, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        })
        .await;

        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Auction has ended\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_no_signer_approval() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());

        with_lock(&mut mock_cache, |cache| {
            cache.expect_is_connected().returning(|| true);
            cache.expect_connect().returning(|| Ok(()));
            cache.expect_ping().returning(|| Ok(()));
            cache
                .expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| Ok(None));
            cache.expect_get_synced_block().returning(|_, _| Ok(150));
            cache.expect_get_auction().returning(|_, _, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        })
        .await;

        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "{\"id\":null,\"cid\":null,\"error\":\"Signer has not approved the settlement contract\"}"),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_approval_too_low() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());

        with_lock(&mut mock_cache, |cache| {
            cache.expect_is_connected().returning(|| true);
            cache.expect_connect().returning(|| Ok(()));
            cache.expect_ping().returning(|| Ok(()));
            cache
                .expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| {
                    Ok(Some((
                        500000000000000000u128.into(),
                        100000000000000000u128.into(),
                    )))
                });
            cache.expect_get_synced_block().returning(|_, _| Ok(150));
            cache.expect_get_auction().returning(|_, _, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        })
        .await;

        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Signer approval amount is insufficient\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_balance_too_low() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());

        with_lock(&mut mock_cache, |cache| {
            cache.expect_is_connected().returning(|| true);
            cache.expect_connect().returning(|| Ok(()));
            cache.expect_ping().returning(|| Ok(()));
            cache
                .expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| {
                    Ok(Some((
                        200000000000000000000u128.into(),
                        100000000000000000u128.into(),
                    )))
                });
            cache.expect_get_synced_block().returning(|_, _| Ok(150));
            cache.expect_get_auction().returning(|_, _, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        })
        .await;

        let mut mock_db = Mutex::new(MockDatabase::new());
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        match response.body() {
            Body::Text(msg) => assert_eq!(
                msg,
                "{\"id\":null,\"cid\":null,\"error\":\"Signer token balance is insufficient\"}"
            ),
            _ => panic!("Malformed response"),
        }
    }

    #[tokio::test]
    async fn request_handler_sig_happy_path() {
        let bid_payload = dummy_data::new_bid_payload(dummy_data::BidPayloadOption::Valid);
        let mut r = Request::new(Body::from(to_string(&bid_payload).unwrap()));
        *r.method_mut() = Method::PUT;
        let mut mock_cache = Mutex::new(MockCache::new());

        with_lock(&mut mock_cache, |cache| {
            cache.expect_is_connected().returning(|| true);
            cache.expect_connect().returning(|| Ok(()));
            cache.expect_ping().returning(|| Ok(()));
            cache
                .expect_get_signer_approve_and_bal_amts()
                .returning(|_, _, _| {
                    Ok(Some((
                        200000000000000000000u128.into(),
                        200000000000000000000u128.into(),
                    )))
                });
            cache.expect_get_synced_block().returning(|_, _| Ok(150));
            cache.expect_get_auction().returning(|_, _, _| {
                Ok(Some(dummy_data::new_auction(
                    dummy_data::AuctionOption::Valid,
                )))
            });
        })
        .await;

        let mut mock_db = Mutex::new(MockDatabase::new());
        with_lock(&mut mock_db, |db| {
            db.expect_is_connected().returning(|| true);
            db.expect_connect().returning(|| Ok(()));
            db.expect_ping().returning(|| Ok(()));
            db.expect_insert_bid()
                .returning(|_| Ok("0xsomehash".to_string()));
        })
        .await;
        let response = put_request_handler(r, &mut mock_cache, &mut mock_db)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "{\"id\":\"0xsomehash\",\"cid\":\"bafkreicmgtpekgtrzmcahkzcpcrkalfxagfw2crbt4kqts7fxsrexjxzna\",\"error\":null}"),
            _ => panic!("Malformed response"),
        }
    }
}
