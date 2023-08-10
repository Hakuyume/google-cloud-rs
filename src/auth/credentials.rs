mod authorized_user;
mod external_account;

use super::Token;
use crate::Error;
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Credentials {
    AuthorizedUser(authorized_user::AuthorizedUser),
    ExternalAccount(external_account::ExternalAccount),
}

impl Credentials {
    #[tracing::instrument(err, ret)]
    pub fn from_env() -> Result<Option<Self>, Error> {
        if let Ok(path) = env::var("GOOGLE_APPLICATION_CREDENTIALS") {
            tracing::debug!("loading credentials from {}", path);
            Ok(serde_json::from_slice(&fs::read(path)?)?)
        } else {
            tracing::warn!("GOOGLE_APPLICATION_CREDENTIALS is unset");
            Ok(None)
        }
    }

    #[tracing::instrument(err, ret, skip(client))]
    pub async fn refresh(&self, client: &reqwest::Client, scopes: &[&str]) -> Result<Token, Error> {
        match &self {
            Self::AuthorizedUser(credentials) => credentials.refresh(client, scopes).await,
            Self::ExternalAccount(credentials) => credentials.refresh(client, scopes).await,
        }
    }
}
