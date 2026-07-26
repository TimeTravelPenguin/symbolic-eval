//! CBOR (de)serialization helpers used to exchange data with the Typst host.
//!
//! Typst plugins communicate over raw byte buffers, so plugin arguments and
//! results are encoded as CBOR. These thin wrappers map the underlying
//! [`ciborium`] errors onto [`SymbolicEvalError::Cbor`].

use ciborium::{de::from_reader, ser::into_writer};

use crate::error::SymbolicEvalError;

/// Decodes a CBOR byte buffer into a value of type `T`.
///
/// # Errors
///
/// Returns [`SymbolicEvalError::Cbor`] if `bytes` is not valid CBOR or does not
/// match the shape of `T`.
pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SymbolicEvalError> {
    from_reader(bytes).map_err(|err| SymbolicEvalError::Cbor(err.to_string()))
}

/// Encodes a value as a CBOR byte buffer.
///
/// # Errors
///
/// Returns [`SymbolicEvalError::Cbor`] if `value` cannot be serialized.
pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SymbolicEvalError> {
    let mut out = Vec::new();
    into_writer(value, &mut out).map_err(|err| SymbolicEvalError::Cbor(err.to_string()))?;

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{PluginArgsExpressions, PluginArgsFunction};

    #[test]
    fn encode_then_decode_roundtrips() {
        let args = PluginArgsExpressions {
            exprs: vec!["x^2 + y".to_string()],
            params: vec!["x".to_string(), "y".to_string()],
            functions: vec![PluginArgsFunction {
                name: "f".to_string(),
                args: vec!["z".to_string()],
                body: "z * z".to_string(),
            }],
        };

        let bytes = encode(&args).unwrap();
        let decoded: PluginArgsExpressions = decode(&bytes).unwrap();

        assert_eq!(args, decoded);
    }

    #[test]
    fn decode_rejects_invalid_bytes() {
        let err = decode::<PluginArgsExpressions>(&[0xff, 0x00, 0x13]).unwrap_err();

        assert!(matches!(err, SymbolicEvalError::Cbor(_)));
    }
}
