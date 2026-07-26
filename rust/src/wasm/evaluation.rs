use crate::codec::{decode, encode};
use crate::error::SymbolicEvalError;
use crate::expressions::{Expressions, SymbolDomain};
use crate::{PluginArgsExpressions, evaluation};

use crate::expressions::Function;

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
