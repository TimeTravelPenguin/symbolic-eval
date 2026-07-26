//! CBOR (de)serialization helpers used to exchange data with the Typst host.
//!
//! Typst plugins communicate over raw byte buffers, so plugin arguments and
//! results are encoded as CBOR. These thin wrappers map the underlying
//! [`ciborium`] errors onto [`SymbolicEvalError::Cbor`].

use ciborium::{de::from_reader, ser::into_writer};

use crate::error::SymbolicEvalError;

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SymbolicEvalError> {
    from_reader(bytes).map_err(|err| SymbolicEvalError::Cbor(err.to_string()))
}

pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SymbolicEvalError> {
    let mut out = Vec::new();
    into_writer(value, &mut out).map_err(|err| SymbolicEvalError::Cbor(err.to_string()))?;

    Ok(out)
}
