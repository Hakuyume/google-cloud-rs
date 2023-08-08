use chrono::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let manager = google_cloud::auth::Manager::from_env(reqwest::Client::new())?;
    let token = manager
        .refresh(
            &["https://www.googleapis.com/auth/cloud-platform"],
            Duration::minutes(5),
        )
        .await?;
    println!("{}", token.access_token);

    Ok(())
}
