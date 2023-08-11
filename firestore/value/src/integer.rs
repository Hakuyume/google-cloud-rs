use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(
    bound(deserialize = "T: TryFrom<i64>, T::Error: Display"),
    try_from = "Inner"
)]
pub struct Integer<T>(pub T);

#[serde_with::serde_as]
#[derive(Deserialize, Serialize)]
struct Inner {
    #[serde(rename = "integerValue")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    value: i64,
}

impl<T> Serialize for Integer<T>
where
    T: Copy,
    i64: TryFrom<T>,
    <i64 as TryFrom<T>>::Error: Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Inner {
            value: self
                .0
                .try_into()
                .map_err(<S::Error as serde::ser::Error>::custom)?,
        }
        .serialize(serializer)
    }
}

impl<T> TryFrom<Inner> for Integer<T>
where
    T: TryFrom<i64>,
{
    type Error = T::Error;
    fn try_from(value: Inner) -> Result<Self, Self::Error> {
        value.value.try_into().map(Self)
    }
}
