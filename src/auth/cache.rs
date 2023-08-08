use super::Token;
use chrono::{Duration, Utc};
use std::collections::HashMap;

#[derive(Default)]
pub struct Cache {
    map: HashMap<Box<[Box<str>]>, Token>,
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, scopes: &[&str], lifetime: Duration) -> Option<&Token> {
        let now = Utc::now();
        if let Some(token) = self.map.get(
            &scopes
                .into_iter()
                .copied()
                .map(Box::from)
                .collect::<Box<_>>(),
        ) {
            (token.expires_at > now + lifetime).then_some(token)
        } else {
            None
        }
    }

    pub fn put(&mut self, scopes: &[&str], token: Token) {
        self.map.insert(
            scopes
                .into_iter()
                .copied()
                .map(Box::from)
                .collect::<Box<_>>(),
            token,
        );
        let now = Utc::now();
        self.map.retain(|_, token| token.expires_at > now);
    }
}
