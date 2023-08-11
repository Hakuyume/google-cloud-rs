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
        bytes: super::Bytes<&'a [u8]>,
        integer: super::Integer<u8>,
        reference: super::Reference<&'a str>,
        string: super::String<&'a str>,
    }

    assert_eq!(
        serde_json::to_value(&S {
            bytes: super::Bytes(b"bytes"),
            integer: super::Integer(42),
            reference: super::Reference(
                "projects/{project_id}/databases/{database_id}/documents/{document_path}",
            ),
            string: super::String("string"),
        })
        .unwrap(),
        sample(),
    );
}

#[test]
fn test_deserialize() {
    #[derive(Deserialize)]
    struct D {
        bytes: super::Bytes<Vec<u8>>,
        integer: super::Integer<u8>,
        reference: super::Reference<String>,
        string: super::String<String>,
    }

    let d = serde_json::from_value::<D>(sample()).unwrap();
    assert_eq!(d.bytes.0, b"bytes");
    assert_eq!(d.integer.0, 42);
    assert_eq!(
        d.reference.0,
        "projects/{project_id}/databases/{database_id}/documents/{document_path}",
    );
    assert_eq!(d.string.0, "string");
}
