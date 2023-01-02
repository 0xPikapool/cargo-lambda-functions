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
        let queue_url = match env::var("SQS_URL") {
            Ok(url) => url,
            Err(e) => return Err(format!("Error getting SQS_URL: {}", e)),
        };

        let request = SendMessageRequest {
            message_body: message_body.to_string(),
            queue_url,
            ..Default::default()
        };
        let client = SqsClient::new(Region::UsEast1);
        match client.send_message(request).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error sending message: {}", e)),
        }
    }
}
