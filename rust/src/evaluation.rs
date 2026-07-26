use itertools::Itertools;

use crate::SymbolicEvalError;
use crate::expressions::{Expressions, SymbolDomain};

pub type EvaluationResult = Vec<(Vec<f64>, Vec<f64>)>;

/// Evaluates a single symbolic expression over a specified domain for a given symbol.
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
