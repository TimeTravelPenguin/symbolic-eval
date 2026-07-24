mod codec;
mod error;
mod evaluation;

use codec::{decode, encode};
pub use error::SymbolicEvalError;

use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::{initiate_protocol, wasm_func};

#[cfg(target_arch = "wasm32")]
initiate_protocol!();

#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU64, Ordering};

use crate::evaluation::{EvaluationArgs, Function, SymbolDomain};

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> Result<(), getrandom::Error> {
    static STATE: AtomicU64 = AtomicU64::new(0x4d59_5df4_d0f3_3173);

    let mut state = STATE
        .fetch_add(0x9e37_79b9_7f4a_7c15, Ordering::Relaxed)
        .wrapping_add(len as u64);
    let bytes = unsafe { std::slice::from_raw_parts_mut(dest, len) };

    for byte in bytes {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        *byte = state.wrapping_mul(0x2545_f491_4f6c_dd1d) as u8;
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginArgsFunction {
    pub name: String,
    pub args: Vec<String>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginArgs {
    pub exprs: Vec<String>,
    pub params: Vec<String>,
    pub functions: Vec<PluginArgsFunction>,
    pub domains: Vec<SymbolDomain>,
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn eval_expr(args: &[u8]) -> Result<Vec<u8>, SymbolicEvalError> {
    let args: PluginArgs = decode(args)?;

    let functions = args
        .functions
        .iter()
        .map(|f| Function::new(&f.name, &f.args, &f.body))
        .collect::<Result<Vec<_>, _>>()?;

    let args = EvaluationArgs::new(&args.exprs, &args.params, &functions, &args.domains)?;
    let results = evaluation::eval_exprs(args)?;

    encode(&results)
}
