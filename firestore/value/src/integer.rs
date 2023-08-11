use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;

pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Clone,
    i64: TryFrom<T>,
    <i64 as TryFrom<T>>::Error: Display,
    S: Serializer,
{
    Inner {
        value: value
            .clone()
            .try_into()
            .map_err(<S::Error as serde::ser::Error>::custom)?,
    }
    .serialize(serializer)
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: TryFrom<i64>,
    T::Error: Display,
    D: Deserializer<'de>,
{
    Inner::deserialize(deserializer)?
        .value
        .try_into()
        .map_err(<D::Error as serde::de::Error>::custom)
}

#[serde_with::serde_as]
#[derive(Deserialize, Serialize)]
struct Inner {
    #[serde(rename = "integerValue")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    value: i64,
}
