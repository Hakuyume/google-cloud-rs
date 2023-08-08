pub mod cache;
pub mod credentials;

use crate::Error;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("access_token", &crate::SENSITIVE)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[derive(Clone)]
pub struct Manager {
    client: reqwest::Client,
    credentials: Arc<credentials::Credentials>,
    cache: Arc<RwLock<cache::Cache>>,
}

impl Manager {
    #[tracing::instrument(skip_all)]
    pub fn from_env(client: reqwest::Client) -> Result<Self, Error> {
        Ok(Self {
            client,
            credentials: Arc::new(
                credentials::Credentials::from_env()?.ok_or_else(|| Error::NoAuthProvider)?,
            ),
            cache: Default::default(),
        })
    }

    pub async fn refresh(&self, scopes: &[&str], lifetime: Duration) -> Result<Token, Error> {
        let token = self.cache.read().await.get(scopes, lifetime).cloned();
        if let Some(token) = token {
            Ok(token)
        } else {
            let token = self.credentials.refresh(&self.client, scopes).await?;
            self.cache.write().await.put(scopes, token.clone());
            Ok(token)
        }
    }
}
