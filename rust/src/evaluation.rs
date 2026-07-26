//! Sampling expressions over a Cartesian grid of parameter domains.

use itertools::Itertools;

use crate::SymbolicEvalError;
use crate::expressions::{Expressions, SymbolDomain};

/// The result of [`eval_exprs`]: one `(inputs, outputs)` pair per grid point.
///
/// `inputs` holds the parameter values at that point (one per domain, in
/// parameter order) and `outputs` holds the value of each expression there.
pub type EvaluationResult = Vec<(Vec<f64>, Vec<f64>)>;

/// Evaluates every expression in `exprs` at every point of the Cartesian
/// product of the given parameter `domains`.
///
/// There must be one [`SymbolDomain`] per free parameter, given in the same
/// order as [`Expressions::params`]. Each domain is sampled at
/// [`samples`](SymbolDomain::samples) evenly-spaced points; the first and last
/// samples are pinned to `min` and `max` exactly to avoid floating-point drift
/// at the endpoints. The total number of grid points (and result rows) is the
/// product of all `samples`.
///
/// # Errors
///
/// Returns an error if the expressions cannot be compiled into an evaluator.
pub fn eval_exprs(
    exprs: Expressions,
    domains: Vec<SymbolDomain>,
) -> Result<EvaluationResult, SymbolicEvalError> {
    let mut ev = exprs.evaluator()?;

    let total_samples: usize = domains.iter().map(|d| d.samples).product();
    let mut results = Vec::with_capacity(total_samples);

    let domains = domains.iter().map(|domain| {
        let step = if domain.samples > 1 {
            (domain.max - domain.min) / (domain.samples - 1) as f64
        } else {
            0.0
        };

        (0..domain.samples).map(move |idx| {
            if idx == 0 {
                domain.min
            } else if idx + 1 == domain.samples {
                domain.max
            } else {
                domain.min + step * idx as f64
            }
        })
    });

    let num_exprs = exprs.exprs.len();
    for inputs in domains.multi_cartesian_product() {
        let mut out = vec![0.0; num_exprs];

        ev.evaluate(&inputs, &mut out);
        results.push((inputs, out));
    }

    Ok(results)
}
