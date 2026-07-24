use itertools::Itertools;
use serde::{Deserialize, Serialize};
use symbolica::prelude::*;

use crate::{SymbolicEvalError, error::SymbolicaError};

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

impl Function {
    pub fn new(
        name: impl AsRef<str>,
        args: &[impl AsRef<str>],
        body: impl AsRef<str>,
    ) -> Result<Self, SymbolicEvalError> {
        let name = try_symbol!(name.as_ref()).map_err(|s| SymbolicaError::Symbol {
            input: s.to_string(),
            message: "Failed to parse function name".to_string(),
        })?;

        let args = parse_symbols(
            &args
                .iter()
                .map(|s| s.as_ref().to_string())
                .collect::<Vec<_>>(),
        )?;

        let body = try_parse!(body).map_err(|s| SymbolicaError::Parse {
            input: s.to_string(),
            message: "Failed to parse function body".to_string(),
        })?;

        Ok(Function { name, args, body })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationArgs {
    pub exprs: Vec<Atom>,
    pub params: Vec<Atom>,
    pub functions: Vec<Function>,
    pub domains: Vec<SymbolDomain>,
}

impl EvaluationArgs {
    pub fn new(
        exprs: &[impl AsRef<str>],
        params: &[impl AsRef<str>],
        functions: &[Function],
        domains: &[SymbolDomain],
    ) -> Result<Self, SymbolicEvalError> {
        if exprs.is_empty() {
            return Err(SymbolicEvalError::ArgumentError(
                "No expressions provided".to_string(),
            ));
        }

        if domains.is_empty() {
            return Err(SymbolicEvalError::ArgumentError(
                "No domains provided".to_string(),
            ));
        }

        if params.len() != domains.len() {
            return Err(SymbolicEvalError::ArgumentError(
                "Number of parameters and domains must match".to_string(),
            ));
        }

        let exprs = parse_exprs(
            &exprs
                .iter()
                .map(|s| s.as_ref().to_string())
                .collect::<Vec<_>>(),
        )?;

        let params = parse_exprs(
            &params
                .iter()
                .map(|s| s.as_ref().to_string())
                .collect::<Vec<_>>(),
        )?;

        Ok(EvaluationArgs {
            exprs,
            params,
            functions: functions.to_vec(),
            domains: domains.to_vec(),
        })
    }
}

pub type EvaluationResult = Vec<(Vec<f64>, Vec<f64>)>;

/// Evaluates a single symbolic expression over a specified domain for a given symbol.
pub fn eval_exprs(eval_args: EvaluationArgs) -> Result<EvaluationResult, SymbolicaError> {
    let mut ev = Atom::evaluator_multiple(&eval_args.exprs, &eval_args.params);

    for f in eval_args.functions {
        ev = ev.add_function(f.name, f.args, f.body)?;
    }

    let mut ev = ev.build()?.map_coeff(&|c| c.re.to_f64());

    let total_samples: usize = eval_args.domains.iter().map(|d| d.samples).product();
    let mut results = Vec::with_capacity(total_samples);

    let domains = eval_args.domains.iter().map(|domain| {
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

    for inputs in domains.multi_cartesian_product() {
        let mut out = vec![0.0; eval_args.exprs.len()];

        ev.evaluate(&inputs, &mut out);
        results.push((inputs, out));
    }

    Ok(results)
}
