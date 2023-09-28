mod cache;
mod client;
mod credentials;
mod error;
mod token;

pub use client::Client;
pub use error::Error;
pub use token::Token;

pub const DEFAULT_SCOPES: &[&str] = &["https://www.googleapis.com/auth/cloud-platform"];
const SENSITIVE: &str = "***";
