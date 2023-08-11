use serde::{Deserialize, Serialize, Serializer};

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(bound(deserialize = "T: TryFrom<Vec<u8>>"), try_from = "Inner<T>")]
pub struct Bytes<T>(pub T);

#[serde_with::serde_as]
#[derive(Deserialize, Serialize)]
#[serde(bound(deserialize = "T: TryFrom<Vec<u8>>", serialize = "T: AsRef<[u8]>"))]
struct Inner<T> {
    #[serde(rename = "bytesValue")]
    #[serde_as(as = "serde_with::base64::Base64")]
    value: T,
}

impl<T> Serialize for Bytes<T>
where
    T: AsRef<[u8]>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Inner { value: &self.0 }.serialize(serializer)
    }
}

impl<T> From<Inner<T>> for Bytes<T> {
    fn from(value: Inner<T>) -> Self {
        Self(value.value)
    }
}
