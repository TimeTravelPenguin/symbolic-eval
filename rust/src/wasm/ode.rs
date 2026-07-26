use crate::PluginArgsExpressions;
use crate::codec::{decode, encode};
use crate::error::SymbolicEvalError;
use crate::expressions::Expressions;
use crate::ode::{self, OdeConfig};

use crate::expressions::Function;

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn eval_ode(args: &[u8], ode_config: &[u8]) -> Result<Vec<u8>, SymbolicEvalError> {
    let args: PluginArgsExpressions = decode(args)?;
    let config: OdeConfig = decode(ode_config)?;

    let functions = args
        .functions
        .iter()
        .map(|f| Function::new(&f.name, &f.args, &f.body))
        .collect::<Result<Vec<_>, _>>()?;

    let exprs = Expressions::new(
        &args.exprs,
        &args.params,
        &functions,
        Expressions::default_constants(),
    )?;

    let result = ode::eval_ode(exprs, config)?;

    encode(&result)
}
