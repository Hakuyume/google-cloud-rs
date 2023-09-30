use chrono::Duration;
use google_cloud_auth::{Client, DEFAULT_SCOPES};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::from_env(http_dispatch::Client::hyper())?;
    let token = client.refresh(DEFAULT_SCOPES, Duration::minutes(5)).await?;
    println!("{}", token.access_token);

    Ok(())
}
