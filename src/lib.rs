pub mod auth;

use bytes::Bytes;
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

    #[error("[{status:?}] {body:?}")]
    Http {
        status: reqwest::StatusCode,
        body: Bytes,
    },
    #[error("no auth provider found")]
    NoAuthProvider,
}

impl Error {
    async fn check_response(response: reqwest::Response) -> Result<reqwest::Response, Self> {
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Self::Http {
                status: response.status(),
                body: response.bytes().await?,
            })
        }
    }
}
