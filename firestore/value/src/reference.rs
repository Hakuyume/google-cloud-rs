use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(bound(deserialize = "T: FromStr, T::Err: Display"), from = "Inner<T>")]
pub struct Reference<T>(pub T);

#[serde_with::serde_as]
#[derive(Deserialize, Serialize)]
#[serde(bound(deserialize = "T: FromStr, T::Err: Display", serialize = "T: Display"))]
struct Inner<T> {
    #[serde(rename = "referenceValue")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    value: T,
}

impl<T> Serialize for Reference<T>
where
    T: Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Inner { value: &self.0 }.serialize(serializer)
    }
}

impl<T> From<Inner<T>> for Reference<T> {
    fn from(value: Inner<T>) -> Self {
        Self(value.value)
    }
}
