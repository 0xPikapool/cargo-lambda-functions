use async_trait::async_trait;
use mockall::automock;
use rusoto_core::Region;
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use std::env;

#[automock]
#[async_trait]
pub trait Sink: Send {
    async fn send(&mut self, message: &str) -> Result<(), String>;
}

pub struct SqsProvider {}

#[async_trait]
impl Sink for SqsProvider {
    async fn send(&mut self, message_body: &str) -> Result<(), String> {
        let queue_url = env::var("SQS_URL").unwrap();

        let request = SendMessageRequest {
            message_body: message_body.to_string(),
            queue_url,
            ..Default::default()
        };
        let client = SqsClient::new(Region::UsEast1);
        let result = client.send_message(request).await;

        println!("{:?}", result);

        Ok(())
    }
}
