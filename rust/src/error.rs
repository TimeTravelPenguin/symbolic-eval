use thiserror::Error;

#[derive(Debug, Error)]
pub enum SymbolicEvalError {
    #[error("CBOR error: {0}")]
    Cbor(String),
    #[error(transparent)]
    SymbolicaError(#[from] SymbolicaError),
    #[error("Argument error: {0}")]
    ArgumentError(String),
}

#[derive(Debug, Error)]
pub enum SymbolicaError {
    #[error("Evaluation error: {0}")]
    Evaluation(#[from] symbolica::evaluate::EvaluationError),
    #[error("Parse error: Error parsing input '{0}': {1}", .input, .message)]
    Parse { input: String, message: String },
    #[error("Symbol error: Error parsing symbol '{0}': {1}", .input, .message)]
    Symbol { input: String, message: String },
    #[error("Function error: Error parsing function '{0}': {1}", .input, .message)]
    Function { input: String, message: String },
}
