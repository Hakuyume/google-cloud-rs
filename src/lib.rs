pub mod auth;

use std::io;

const SENSITIVE: &str = "***";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

async fn check_response(response: reqwest::Response) -> Result<reqwest::Response, reqwest::Error> {
    if let Err(e) = response.error_for_status_ref() {
        tracing::error!(
            "status = {}, body = {:?}",
            response.status(),
            response.bytes().await?
        );
        Err(e)
    } else {
        Ok(response)
    }
}
