use crate::{cache, credentials, Error, Token};
use chrono::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Client {
    client: http_dispatch::Client,
    credentials: Arc<credentials::Credentials>,
    cache: Arc<RwLock<cache::Cache>>,
}

impl Client {
    #[tracing::instrument(err, level = "debug", skip(client))]
    pub fn from_env(client: http_dispatch::Client) -> Result<Self, Error> {
        Ok(Self {
            client,
            credentials: Arc::new(
                credentials::Credentials::from_env()?
                    .ok_or_else(|| Error::TokenProviderNotFound)?,
            ),
            cache: Default::default(),
        })
    }

    #[tracing::instrument(err, level = "debug", ret, skip(self))]
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
