use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;

pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    Inner { value }.serialize(serializer)
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    Ok(Inner::deserialize(deserializer)?.value)
}

#[serde_with::serde_as]
#[derive(Deserialize, Serialize)]
#[serde(bound(deserialize = "T: FromStr, T::Err: Display", serialize = "T: Display"))]
struct Inner<T> {
    #[serde(rename = "referenceValue")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    value: T,
}
