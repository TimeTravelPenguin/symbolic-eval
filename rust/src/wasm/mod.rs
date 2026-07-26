//! The plugin's WebAssembly entry points, as called from Typst.
//!
//! Each function here takes and returns raw CBOR byte buffers (the only types
//! Typst plugins can exchange). On the `wasm32` target they are additionally
//! annotated with `#[wasm_func]` to conform to the
//! [`wasm-minimal-protocol`](wasm_minimal_protocol) ABI; on native targets they
//! remain ordinary functions so they can be exercised by tests and examples.

pub mod evaluation;
pub mod ode;
