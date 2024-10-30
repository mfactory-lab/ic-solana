//! Wrapper for serde_json::Value that implements Candid serialization as String.

use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Wrapper for serde_json::Value that implements Candid serialization as String.
/// Note: in did file, this type is represented as `text`, thus you could also send a string
/// directly to the method that expects `CandidValue`, without wrapping it in `CandidValue`.
pub struct CandidValue(pub serde_json::Value);

impl From<CandidValue> for serde_json::Value {
    fn from(value: CandidValue) -> Self {
        value.0
    }
}

impl From<serde_json::Value> for CandidValue {
    fn from(value: serde_json::Value) -> Self {
        CandidValue(value)
    }
}

impl CandidType for CandidValue {
    fn id() -> candid::types::TypeId {
        String::id()
    }

    fn _ty() -> candid::types::Type {
        String::_ty()
    }
    // only serialize the value encoding
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        let s = serde_json::to_string(&self.0).map_err(serde::ser::Error::custom)?;

        s.idl_serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use candid::{Decode, Encode};

    use super::*;

    #[test]
    fn test_candid_value() {
        let value = CandidValue(serde_json::json!({"key": "value"}));
        let json_encoded = serde_json::to_string(&value).unwrap();

        assert_eq!(json_encoded, r#"{"key":"value"}"#);

        let _json_decoded = serde_json::from_str::<CandidValue>(&json_encoded).unwrap();
        let candid_encoded = Encode!(&value).unwrap();
        let _candid_decoded = Decode!(&candid_encoded, CandidValue).unwrap();

        let value = CandidValue(serde_json::json!({"outer_field": {"key": "value"}}));
        let json_encoded = serde_json::to_string(&value).unwrap();

        assert_eq!(json_encoded, r#"{"outer_field":{"key":"value"}}"#);

        let _json_decoded = serde_json::from_str::<CandidValue>(&json_encoded).unwrap();
        let candid_encoded = Encode!(&value).unwrap();
        let _candid_decoded = Decode!(&candid_encoded, CandidValue).unwrap();
    }

    #[test]
    fn test_from_text() {
        let json_string = r#"{"key":"value"}"#;
        let candid_encoded = Encode!(&json_string).unwrap();
        let _candid_decoded = Decode!(&candid_encoded, CandidValue).unwrap();
    }
}
