use chrono::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let manager = google_cloud::auth::Manager::from_env(reqwest::Client::new())?;
    let token = manager
        .refresh(google_cloud::auth::DEFAULT_SCOPES, Duration::minutes(5))
        .await?;
    println!("{}", token.access_token);

    Ok(())
}
