use thiserror::Error;

#[derive(Debug, Error)]
pub enum SymbolicEvalError {
    #[error("CBOR error: {0}")]
    Cbor(String),
}
