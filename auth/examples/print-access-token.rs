use chrono::Duration;
use google_cloud_auth::{Manager, DEFAULT_SCOPES};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let manager = Manager::from_env(reqwest::Client::new())?;
    let token = manager
        .refresh(DEFAULT_SCOPES, Duration::minutes(5))
        .await?;
    println!("{}", token.access_token);

    Ok(())
}
