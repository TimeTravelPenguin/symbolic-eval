mod codec;
mod error;

use codec::{decode, encode};
pub use error::SymbolicEvalError;

use symbolica::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::{initiate_protocol, wasm_func};

#[cfg(target_arch = "wasm32")]
initiate_protocol!();

#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> std::result::Result<(), getrandom::Error> {
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

type Result = std::result::Result<Vec<u8>, SymbolicEvalError>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SymbolDomain {
    pub min: f64,
    pub max: f64,
    pub samples: usize,
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn eval_expr(expr: &[u8], symbol: &[u8], domain: &[u8]) -> Result {
    let expr: String = decode(expr)?;
    let symbol: String = decode(symbol)?;
    let domain: SymbolDomain = decode(domain)?;

    let params = [parse!(symbol)];

    let mut ev = parse!(expr)
        .evaluator(&params)
        .build()
        .unwrap()
        .map_coeff(&|c| c.re.to_f64());

    let mut results = Vec::with_capacity(domain.samples);
    let step = (domain.max - domain.min) / (domain.samples as f64 - 1.0);

    for i in 0..domain.samples {
        let x = domain.min + step * (i as f64);
        let mut out = [0.0];

        ev.evaluate(&[x], &mut out);
        results.push((x, out[0]));
    }

    encode(&results)
}
