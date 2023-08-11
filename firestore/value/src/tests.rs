use serde::{Deserialize, Serialize};

fn sample() -> serde_json::Value {
    serde_json::json!({
        "bytes": {
            "bytesValue": "Ynl0ZXM=",
        },
        "integer": {
            "integerValue": "42",
        },
        "reference": {
            "referenceValue": "projects/{project_id}/databases/{database_id}/documents/{document_path}",
        },
        "string": {
            "stringValue": "string",
        },
    })
}

#[test]
fn test_serialize() {
    #[derive(Serialize)]
    struct S<'a> {
        #[serde(with = "super::bytes")]
        bytes: &'a [u8],
        #[serde(with = "super::integer")]
        integer: u8,
        #[serde(with = "super::reference")]
        reference: &'a str,
        #[serde(with = "super::string")]
        string: &'a str,
    }

    assert_eq!(
        serde_json::to_value(&S {
            bytes: b"bytes",
            integer: 42,
            reference: "projects/{project_id}/databases/{database_id}/documents/{document_path}",
            string: "string",
        })
        .unwrap(),
        sample(),
    );
}

#[test]
fn test_deserialize() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct S {
        #[serde(with = "super::bytes")]
        bytes: Vec<u8>,
        #[serde(with = "super::integer")]
        integer: u8,
        #[serde(with = "super::reference")]
        reference: String,
        #[serde(with = "super::string")]
        string: String,
    }

    assert_eq!(
        serde_json::from_value::<S>(sample()).unwrap(),
        S {
            bytes: b"bytes".to_vec(),
            integer: 42,
            reference: "projects/{project_id}/databases/{database_id}/documents/{document_path}"
                .to_owned(),
            string: "string".to_owned(),
        }
    );
}
