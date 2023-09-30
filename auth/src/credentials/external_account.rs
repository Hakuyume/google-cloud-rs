// https://google.aip.dev/auth/4117

use crate::{Error, Token};
use chrono::{DateTime, Duration, Utc};
use http_dispatch::http::{Method, Uri};
use serde::{Deserialize, Serialize};
use serde_with::formats::SpaceSeparator;
use serde_with::StringWithSeparator;
use tokio::fs;

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
pub struct ExternalAccount {
    pub audience: String,
    pub subject_token_type: String,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub service_account_impersonation_url: Option<Uri>,
    // TODO: service_account_impersonation
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub token_url: Uri,
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
    #[tracing::instrument(err, level = "debug", ret, skip(client))]
    pub async fn refresh(
        &self,
        client: &http_dispatch::Client,
        scopes: &[&str],
    ) -> Result<Token, Error> {
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

            client
                .send::<_, http_dispatch::Json<Response>>((
                    Method::POST,
                    self.token_url.clone(),
                    http_dispatch::Json(Request {
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
                    }),
                ))
                .await?
                .0
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

                client
                    .send::<_, http_dispatch::Json<Response>>((
                        Method::POST,
                        service_account_impersonation_url.clone(),
                        http_dispatch::TypedHeader(
                            http_dispatch::headers::Authorization::bearer(&response.access_token)
                                .unwrap(),
                        ),
                        http_dispatch::Json(Request {
                            delegates: None,
                            scope: scopes,
                            lifetime: Some("3600s"),
                        }),
                    ))
                    .await?
                    .0
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
