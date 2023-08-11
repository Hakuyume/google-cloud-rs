use crate::{cache, credentials, Error, Token};
use chrono::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Manager {
    client: reqwest::Client,
    credentials: Arc<credentials::Credentials>,
    cache: Arc<RwLock<cache::Cache>>,
}

impl Manager {
    #[tracing::instrument(err, skip(client))]
    pub fn from_env(client: reqwest::Client) -> Result<Self, Error> {
        Ok(Self {
            client,
            credentials: Arc::new(
                credentials::Credentials::from_env()?.ok_or_else(|| Error::NoAuthProvider)?,
            ),
            cache: Default::default(),
        })
    }

    #[tracing::instrument(err, ret, skip(self))]
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
