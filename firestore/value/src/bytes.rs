use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    Inner { value }.serialize(serializer)
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: TryFrom<Vec<u8>>,
    D: Deserializer<'de>,
{
    Ok(Inner::deserialize(deserializer)?.value)
}

#[serde_with::serde_as]
#[derive(Deserialize, Serialize)]
#[serde(bound(deserialize = "T: TryFrom<Vec<u8>>", serialize = "T: AsRef<[u8]>"))]
struct Inner<T> {
    #[serde(rename = "bytesValue")]
    #[serde_as(as = "serde_with::base64::Base64")]
    value: T,
}
