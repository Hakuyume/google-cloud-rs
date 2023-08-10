// https://google.aip.dev/auth/4117

use super::Token;
use crate::Error;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_with::formats::SpaceSeparator;
use serde_with::StringWithSeparator;
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct ExternalAccount {
    pub audience: String,
    pub subject_token_type: String,
    pub service_account_impersonation_url: Option<String>,
    // TODO: service_account_impersonation
    pub token_url: String,
    pub credential_source: CredentialSource,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CredentialSource {
    File(File),
}

#[derive(Debug, Deserialize)]
pub struct File {
    pub file: String,
}

impl ExternalAccount {
    #[tracing::instrument(err, ret, skip(client))]
    pub async fn refresh(&self, client: &reqwest::Client, scopes: &[&str]) -> Result<Token, Error> {
        let external_credential = match &self.credential_source {
            CredentialSource::File(source) => fs::read_to_string(&source.file).await?,
        };

        let now = Utc::now();
        let response = {
            // https://cloud.google.com/iam/docs/reference/sts/rest/v1/TopLevel/token#request-body
            #[serde_with::serde_as]
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Request<'a> {
                grant_type: &'a str,
                audience: &'a str,
                #[serde_as(as = "StringWithSeparator::<SpaceSeparator, &str>")]
                scope: Vec<&'a str>,
                requested_token_type: &'a str,
                subject_token: &'a str,
                subject_token_type: &'a str,
            }

            // https://cloud.google.com/iam/docs/reference/sts/rest/v1/TopLevel/token#response-body
            #[derive(Deserialize)]
            struct Response {
                access_token: String,
                expires_in: i64,
            }

            crate::check_response(
                client
                    .post(&self.token_url)
                    .json(&Request {
                        grant_type: "urn:ietf:params:oauth:grant-type:token-exchange",
                        audience: &self.audience,
                        requested_token_type: "urn:ietf:params:oauth:token-type:access_token",
                        subject_token_type: &self.subject_token_type,
                        scope: if self.service_account_impersonation_url.is_some() {
                            super::super::DEFAULT_SCOPES
                        } else {
                            scopes
                        }
                        .into(),
                        subject_token: &external_credential,
                    })
                    .send()
                    .await?,
            )
            .await?
            .json::<Response>()
            .await?
        };

        if let Some(service_account_impersonation_url) = &self.service_account_impersonation_url {
            let response = {
                // https://cloud.google.com/iam/docs/reference/credentials/rest/v1/projects.serviceAccounts/generateAccessToken#request-body
                #[serde_with::serde_as]
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Request<'a> {
                    delegates: Option<&'a [&'a str]>,
                    scope: &'a [&'a str],
                    lifetime: Option<&'a str>,
                }

                // https://cloud.google.com/iam/docs/reference/credentials/rest/v1/projects.serviceAccounts/generateAccessToken#response-body
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Response {
                    access_token: String,
                    expire_time: DateTime<Utc>,
                }

                crate::check_response(
                    client
                        .post(service_account_impersonation_url)
                        .bearer_auth(&response.access_token)
                        .json(&Request {
                            delegates: None,
                            scope: scopes,
                            lifetime: Some("3600s"),
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
                expires_at: response.expire_time,
            })
        } else {
            Ok(Token {
                access_token: response.access_token,
                expires_at: now + Duration::seconds(response.expires_in),
            })
        }
    }
}
