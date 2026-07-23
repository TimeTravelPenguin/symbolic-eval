//! CBOR (de)serialization and image decoding helpers.

use std::io::Cursor;

use ciborium::{de::from_reader, ser::into_writer};

use crate::error::SymbolicEvalError;

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SymbolicEvalError> {
    from_reader(Cursor::new(bytes)).map_err(|err| SymbolicEvalError::Cbor(err.to_string()))
}

pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SymbolicEvalError> {
    let mut out = Vec::new();
    into_writer(value, &mut out).map_err(|err| SymbolicEvalError::Cbor(err.to_string()))?;

    Ok(out)
}
