mod cache;
mod credentials;
mod error;
mod manager;
mod token;

pub use error::Error;
pub use manager::Manager;
pub use token::Token;

pub const DEFAULT_SCOPES: &[&str] = &["https://www.googleapis.com/auth/cloud-platform"];
const SENSITIVE: &str = "***";
