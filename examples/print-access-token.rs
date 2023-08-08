use google_cloud::auth::credentials::Credentials;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = reqwest::Client::new();
    let credentials =
        Credentials::from_env()?.ok_or_else(|| anyhow::format_err!("no credentials found"))?;
    let token = credentials
        .refresh(&client, &["https://www.googleapis.com/auth/cloud-platform"])
        .await?;
    println!("{}", token.access_token);

    Ok(())
}
