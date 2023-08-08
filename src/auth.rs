pub mod credentials;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fmt;

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
