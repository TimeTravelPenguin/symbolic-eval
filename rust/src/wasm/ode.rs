use crate::PluginArgsExpressions;
use crate::codec::{decode, encode};
use crate::error::SymbolicEvalError;
use crate::expressions::Expressions;
use crate::ode::{self, OdeConfig};

use crate::expressions::Function;

/// Plugin entry point: integrate a symbolic ODE system.
///
/// `args` is a CBOR-encoded [`PluginArgsExpressions`] whose expressions form
/// the RHS `f(t, y)`, and `ode_config` is a CBOR-encoded
/// [`OdeConfig`](crate::ode::OdeConfig). The result is a CBOR-encoded
/// `Vec<Vec<f64>>` of `[t, y_0, ...]` rows (see [`ode::solve_ode`]).
///
/// # Errors
///
/// Returns an error if either input fails to decode, the expressions fail to
/// parse, or the solver fails.
#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn solve_ode(args: &[u8], ode_config: &[u8]) -> Result<Vec<u8>, SymbolicEvalError> {
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

    let result = ode::solve_ode(exprs, config)?;

    encode(&result)
}
