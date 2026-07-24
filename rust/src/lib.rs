mod codec;
mod error;
mod evaluation;

use codec::{decode, encode};
pub use error::SymbolicEvalError;

use serde::{Deserialize, Serialize};
use symbolica::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::{initiate_protocol, wasm_func};

#[cfg(target_arch = "wasm32")]
initiate_protocol!();

#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    error::SymbolicaError,
    evaluation::{EvaluationArgs, SymbolDomain},
};

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

fn parse_exprs(exprs: &[String]) -> Result<Vec<Atom>, SymbolicEvalError> {
    let result = exprs
        .iter()
        .map(|s| {
            try_parse!(s).map_err(|s| SymbolicaError::Parse {
                input: s.to_string(),
                message: "Failed to parse input".to_string(),
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(result)
}

fn parse_symbols(symbols: &[String]) -> Result<Vec<Symbol>, SymbolicEvalError> {
    let result = symbols
        .iter()
        .map(|s| {
            try_symbol!(s).map_err(|s| SymbolicaError::Symbol {
                input: s.to_string(),
                message: "Failed to parse symbol".to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(result)
}

fn parse_functions(
    functions: &[PluginArgsFunction],
) -> Result<Vec<evaluation::Function>, SymbolicEvalError> {
    let result = functions
        .iter()
        .map(|f| {
            let name = try_symbol!(&f.name).map_err(|s| SymbolicaError::Symbol {
                input: s.to_string(),
                message: "Failed to parse function name".to_string(),
            })?;

            let args = parse_symbols(&f.args)?;

            let body = try_parse!(&f.body).map_err(|s| SymbolicaError::Parse {
                input: s.to_string(),
                message: "Failed to parse function body".to_string(),
            })?;

            Ok(evaluation::Function { name, args, body })
        })
        .collect::<Result<Vec<evaluation::Function>, SymbolicEvalError>>()?;

    Ok(result)
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn eval_expr(args: &[u8]) -> Result<Vec<u8>, SymbolicEvalError> {
    let args: PluginArgs = decode(args)?;

    let args = EvaluationArgs {
        exprs: parse_exprs(&args.exprs)?,
        params: parse_exprs(&args.params)?,
        functions: parse_functions(&args.functions)?,
        domains: args.domains,
    };

    if args.exprs.is_empty() {
        return Err(SymbolicEvalError::ArgumentError(
            "No expressions provided".to_string(),
        ));
    }

    let results = evaluation::eval_exprs(args)?;

    encode(&results)
}
