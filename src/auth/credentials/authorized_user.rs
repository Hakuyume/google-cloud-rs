// https://google.aip.dev/auth/4113

use super::Token;
use crate::Error;
use chrono::{Duration, Utc};
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
    #[tracing::instrument(err, ret, skip(client))]
    pub async fn refresh(&self, client: &reqwest::Client, scopes: &[&str]) -> Result<Token, Error> {
        let now = Utc::now();
        let response = {
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

            #[derive(Deserialize)]
            struct Response {
                access_token: String,
                expires_in: i64,
            }

            Error::check_response(
                client
                    .post("https://oauth2.googleapis.com/token")
                    .json(&Request {
                        grant_type: "refresh_token",
                        client_id: &self.client_id,
                        client_secret: &self.client_secret,
                        refresh_token: &self.refresh_token,
                        scope: scopes.into(),
                    })
                    .send()
                    .await?,
            )
            .await?
            .json::<Response>()
            .await?
        };
        Ok(Token {
            access_token: response.access_token,
            expires_at: now + Duration::seconds(response.expires_in),
        })
    }
}
