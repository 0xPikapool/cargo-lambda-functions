use lambda_http::http::StatusCode;
use lambda_http::{Body, Request};
use pikapool_api::core::request_handler;

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
    async fn request_handler_invalid_signature() {
        let request = Request::new(Body::from("invalid body"));
        let response = request_handler(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        match response.body() {
            Body::Text(msg) => assert_eq!(msg, "expected value at line 1 column 1"),
            _ => panic!("Malformed response"),
        }
    }
}
