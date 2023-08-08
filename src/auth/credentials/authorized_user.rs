use super::Token;
use crate::Error;
use serde::{Deserialize, Serialize};
use serde_with::formats::SpaceSeparator;
use serde_with::StringWithSeparator;
use std::fmt;

#[derive(Deserialize)]
pub struct AuthorizedUser {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
}

impl fmt::Debug for AuthorizedUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorizedUser")
            .field("client_id", &self.client_id)
            .field("client_secret", &crate::SENSITIVE)
            .field("refresh_token", &crate::SENSITIVE)
            .finish()
    }
}

impl AuthorizedUser {
    #[tracing::instrument]
    pub async fn refresh(&self, client: &reqwest::Client, scopes: &[&str]) -> Result<Token, Error> {
        #[serde_with::serde_as]
        #[derive(Serialize)]
        struct Request<'a> {
            grant_type: &'a str,
            client_id: &'a str,
            client_secret: &'a str,
            refresh_token: &'a str,
            #[serde_as(as = "StringWithSeparator::<SpaceSeparator, &str>")]
            scope: Vec<&'a str>,
        }

        let token = client
            .post("https://oauth2.googleapis.com/token")
            .json(&Request {
                grant_type: "refresh_token",
                client_id: &self.client_id,
                client_secret: &self.client_secret,
                refresh_token: &self.refresh_token,
                scope: scopes.into(),
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(token)
    }
}
