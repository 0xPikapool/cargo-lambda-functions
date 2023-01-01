use dotenv;
use env_logger::{self, Target};
use lambda_http::{run, service_fn, Error};
use pikapool_api::core::request_handler;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_default_env()
        .target(Target::Stdout)
        .init();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(request_handler)).await
}
