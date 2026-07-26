use crate::codec::{decode, encode};
use crate::error::SymbolicEvalError;
use crate::expressions::{Expressions, SymbolDomain};
use crate::{PluginArgsExpressions, evaluation};

use crate::expressions::Function;

/// Plugin entry point: evaluate expressions over a grid of parameter domains.
///
/// `args` is a CBOR-encoded [`PluginArgsExpressions`] and `domains` is a
/// CBOR-encoded `Vec<SymbolDomain>`. The result is a CBOR-encoded
/// [`EvaluationResult`](crate::evaluation::EvaluationResult). The
/// [`default_constants`](Expressions::default_constants) (`pi`, `e`, `phi`) are
/// always made available.
///
/// # Errors
///
/// Returns an error if either input fails to decode, any expression or function
/// fails to parse, or evaluation fails.
#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn eval_expr(args: &[u8], domains: &[u8]) -> Result<Vec<u8>, SymbolicEvalError> {
    let args: PluginArgsExpressions = decode(args)?;
    let domains: Vec<SymbolDomain> = decode(domains)?;

    let functions = args
        .functions
        .iter()
        .map(|f| Function::new(&f.name, &f.args, &f.body))
        .collect::<Result<Vec<_>, _>>()?;

    let args = Expressions::new(
        &args.exprs,
        &args.params,
        &functions,
        Expressions::default_constants(),
    )?;

    let results = evaluation::eval_exprs(args, domains)?;
    encode(&results)
}
