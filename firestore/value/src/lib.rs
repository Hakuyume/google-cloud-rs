// https://firebase.google.com/docs/firestore/reference/rest/v1/Value

mod bytes;
mod integer;
mod reference;
mod string;

pub use bytes::Bytes;
pub use integer::Integer;
pub use reference::Reference;
pub use string::String;

#[cfg(test)]
mod tests;
