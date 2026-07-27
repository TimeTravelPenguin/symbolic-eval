//! Sampling expressions over a Cartesian grid of parameter domains.

use itertools::Itertools;
use symbolica::prelude::Complex;

use crate::SymbolicEvalError;
use crate::expressions::{ComplexValue, Expressions, SymbolDomain};

/// The result of [`eval_exprs`]: one `(inputs, outputs)` pair per grid point.
///
/// `inputs` holds the parameter values at that point (one per domain, in
/// parameter order) and `outputs` holds the value of each expression there.
pub type EvaluationResult = Vec<(Vec<f64>, Vec<f64>)>;

/// The complex-capable result of [`eval_complex_exprs`].
///
/// `inputs` holds the parameter values at that point (one per domain, in
/// parameter order) and `outputs` holds the value of each expression there.
pub type ComplexEvaluationResult = Vec<(Vec<ComplexValue>, Vec<ComplexValue>)>;

fn sample_interval(min: f64, max: f64, samples: usize) -> Vec<f64> {
    let step = if samples > 1 {
        (max - min) / (samples - 1) as f64
    } else {
        0.0
    };

    (0..samples)
        .map(|idx| {
            if idx == 0 {
                min
            } else if idx + 1 == samples {
                max
            } else {
                min + step * idx as f64
            }
        })
        .collect()
}

fn sample_domain(domain: &SymbolDomain) -> Vec<ComplexValue> {
    match domain {
        SymbolDomain::RealDomain { min, max, samples } => sample_interval(*min, *max, *samples)
            .into_iter()
            .map(|re| ComplexValue::new(re, 0.0))
            .collect(),

        SymbolDomain::ComplexDomain {
            min_re,
            max_re,
            min_im,
            max_im,
            samples_re,
            samples_im,
        } => {
            let re_values = sample_interval(*min_re, *max_re, *samples_re);
            let im_values = sample_interval(*min_im, *max_im, *samples_im);

            im_values
                .into_iter()
                .cartesian_product(re_values)
                .map(|(im, re)| ComplexValue::new(re, im))
                .collect()
        }
    }
}

pub fn complex_result_to_real(
    result: ComplexEvaluationResult,
) -> Result<EvaluationResult, SymbolicEvalError> {
    result
        .into_iter()
        .map(|(inputs, outputs)| {
            let inputs = inputs
                .into_iter()
                .map(|input| {
                    if input.is_real() {
                        Ok(input.re)
                    } else {
                        Err(SymbolicEvalError::ArgumentError(
                            "cannot convert complex input to real result".to_string(),
                        ))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;

            let outputs = outputs
                .into_iter()
                .map(|output| {
                    if output.is_real() {
                        Ok(output.re)
                    } else {
                        Err(SymbolicEvalError::ArgumentError(
                            "cannot convert complex output to real result".to_string(),
                        ))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok((inputs, outputs))
        })
        .collect()
}

/// Evaluates every expression in `exprs` at every point of the Cartesian
/// product of the given parameter `domains`.
///
/// There must be one real [`SymbolDomain`] per free parameter, given in the
/// same order as [`Expressions::params`]. Each domain is sampled at
/// evenly-spaced points; the first and last samples are pinned to the endpoints
/// exactly to avoid floating-point drift. The total number of grid points (and
/// result rows) is the product of every domain's sample count.
///
/// # Errors
///
/// Returns an error if the expressions cannot be compiled into an evaluator.
pub fn eval_exprs(
    exprs: Expressions,
    domains: Vec<SymbolDomain>,
) -> Result<EvaluationResult, SymbolicEvalError> {
    if domains.iter().any(|domain| !domain.is_real()) {
        return Err(SymbolicEvalError::ArgumentError(
            "eval_exprs only supports real domains; use eval_complex_exprs for complex domains"
                .to_string(),
        ));
    }

    let result = eval_complex_exprs(exprs, domains)?;

    complex_result_to_real(result)
}

/// Evaluates every expression in `exprs` over real or complex parameter
/// domains.
///
/// Real domains produce samples with zero imaginary part. Complex domains sample
/// the imaginary axis outermost and the real axis innermost, so the real part
/// varies fastest in each complex grid.
///
/// # Errors
///
/// Returns an error if the expressions cannot be compiled into an evaluator.
pub fn eval_complex_exprs(
    exprs: Expressions,
    domains: Vec<SymbolDomain>,
) -> Result<ComplexEvaluationResult, SymbolicEvalError> {
    let mut ev = exprs.complex_evaluator()?;

    let domains = domains.iter().map(sample_domain).collect::<Vec<_>>();
    let total_samples: usize = domains.iter().map(Vec::len).product();
    let mut results = Vec::with_capacity(total_samples);

    let num_exprs = exprs.exprs.len();
    for inputs in domains.into_iter().multi_cartesian_product() {
        let params = inputs
            .iter()
            .copied()
            .map(Complex::from)
            .collect::<Vec<_>>();

        let mut out = vec![Complex::new(0.0, 0.0); num_exprs];
        ev.evaluate(&params, &mut out);

        let out = out.into_iter().map(ComplexValue::from).collect();
        results.push((inputs, out));
    }

    Ok(results)
}
