//! Error types for the crate.
//!
//! [`SymbolicEvalError`] is the crate-wide error returned by every fallible
//! public entry point. Errors originating from [`symbolica`] itself are grouped
//! under the nested [`SymbolicaError`].

use thiserror::Error;

/// The top-level error type for all fallible operations in this crate.
#[derive(Debug, Error)]
pub enum SymbolicEvalError {
    /// CBOR encoding or decoding failed. Carries the underlying message from
    /// [`ciborium`].
    #[error("CBOR error: {0}")]
    Cbor(String),
    /// An error surfaced from [`symbolica`] (parsing or evaluation).
    #[error(transparent)]
    SymbolicaError(#[from] SymbolicaError),
    /// The caller supplied invalid arguments (e.g. no expressions to evaluate).
    #[error("Argument error: {0}")]
    ArgumentError(String),
    /// Integrating an ODE system failed.
    #[error(transparent)]
    OdeError(#[from] crate::ode::OdeError),
}

/// Errors produced while parsing or evaluating [`symbolica`] expressions.
///
/// The parse variants keep both the offending `input` and a human-readable
/// `message` so the failure can be reported back to the Typst user precisely.
#[derive(Debug, Error)]
pub enum SymbolicaError {
    /// Numerical evaluation of a compiled expression failed.
    #[error("Evaluation error: {0}")]
    Evaluation(#[from] symbolica::evaluate::EvaluationError),
    /// An expression string could not be parsed into an [`Atom`](symbolica::atom::Atom).
    #[error("Parse error: Error parsing input '{0}': {1}", .input, .message)]
    Parse { input: String, message: String },
    /// A symbol (parameter or constant name) could not be parsed.
    #[error("Symbol error: Error parsing symbol '{0}': {1}", .input, .message)]
    Symbol { input: String, message: String },
    /// A user-defined function could not be parsed.
    #[error("Function error: Error parsing function '{0}': {1}", .input, .message)]
    Function { input: String, message: String },
}
