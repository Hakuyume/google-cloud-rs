pub mod credentials;

use serde::Deserialize;
use std::fmt;

#[derive(Clone, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub expires_in: i64,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("access_token", &crate::SENSITIVE)
            .field("expires_in", &self.expires_in)
            .finish()
    }
}
