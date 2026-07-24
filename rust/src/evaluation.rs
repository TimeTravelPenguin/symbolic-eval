use serde::{Deserialize, Serialize};
use symbolica::prelude::*;

use crate::error::SymbolicaError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolDomain {
    pub min: f64,
    pub max: f64,
    pub samples: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Symbol,
    pub args: Vec<Symbol>,
    pub body: Atom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationArgs {
    pub exprs: Vec<Atom>,
    pub params: Vec<Atom>,
    pub functions: Vec<Function>,
    pub domains: Vec<SymbolDomain>,
}

pub type EvaluationResult = Vec<(Vec<f64>, Vec<f64>)>;

/// Evaluates a single symbolic expression over a specified domain for a given symbol.
pub fn eval_exprs(eval_args: EvaluationArgs) -> Result<EvaluationResult, SymbolicaError> {
    let mut ev = Atom::evaluator_multiple(&eval_args.exprs, &eval_args.params);

    for f in eval_args.functions {
        ev = ev.add_function(f.name, f.args, f.body)?;
    }

    let mut ev = ev.build()?.map_coeff(&|c| c.re.to_f64());

    let domains = eval_args
        .domains
        .iter()
        .map(|d| {
            let step = (d.max - d.min) / (d.samples as f64 - 1.0);
            (0..d.samples).map(move |i| d.min + step * (i as f64))
        })
        .collect::<Vec<_>>();

    let mut results = Vec::with_capacity(
        eval_args.exprs.len() * eval_args.domains.iter().map(|d| d.samples).sum::<usize>(),
    );

    // In order to evaluate the expressions over the Cartesian product of the
    // domains, we need to iterate over all combinations of indices for each
    // domain. We can do this by maintaining a vector of indices, where each
    // index corresponds to the current position in the respective domain's
    // iterator.

    let mut indices = vec![0; domains.len()];
    loop {
        let inputs: Vec<f64> = indices
            .iter()
            .enumerate()
            .map(|(i, &idx)| domains[i].clone().nth(idx).unwrap())
            .collect();

        let mut out = vec![0.0; eval_args.exprs.len()];
        ev.evaluate(&inputs, &mut out);

        results.push((inputs, out));

        // Increment indices
        let mut carry = true;
        for i in (0..indices.len()).rev() {
            if carry {
                indices[i] += 1;
                if indices[i] >= eval_args.domains[i].samples {
                    indices[i] = 0;
                } else {
                    carry = false;
                }
            }
        }

        if carry {
            break;
        }
    }

    Ok(results)
}
