use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use symbolica::prelude::*;

use crate::{SymbolicEvalError, error::SymbolicaError};

fn parse_exprs<I, S, C>(exprs: I) -> Result<C, SymbolicEvalError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    C: FromIterator<Atom>,
{
    let result = exprs
        .into_iter()
        .map(|s| {
            let s = s.as_ref();
            try_parse!(s).map_err(|s| SymbolicaError::Parse {
                input: s.to_string(),
                message: "Failed to parse input".to_string(),
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(result)
}

fn parse_symbols<I, S, C>(symbols: I) -> Result<C, SymbolicEvalError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    C: FromIterator<Symbol>,
{
    let result = symbols
        .into_iter()
        .map(|s| {
            let s = s.as_ref();
            try_symbol!(s).map_err(|s| SymbolicaError::Symbol {
                input: s.to_string(),
                message: "Failed to parse symbol".to_string(),
            })
        })
        .collect::<Result<_, _>>()?;

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
            args.iter()
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
pub struct Expressions {
    pub exprs: Vec<Atom>,
    pub params: Vec<Atom>,
    pub functions: Vec<Function>,
    pub replacements: HashMap<Symbol, Atom>,
}

impl Expressions {
    pub fn new(
        exprs: &[impl AsRef<str>],
        params: &[impl AsRef<str>],
        functions: &[Function],
        constants: &[(impl AsRef<str>, f64)],
    ) -> Result<Self, SymbolicEvalError> {
        if exprs.is_empty() {
            return Err(SymbolicEvalError::ArgumentError(
                "No expressions provided".to_string(),
            ));
        }

        let exprs = parse_exprs(exprs)?;
        let params = parse_exprs(params)?;

        let replacements = constants
            .iter()
            .map(|(s, v)| {
                let symbol = try_symbol!(s.as_ref()).map_err(|s| SymbolicaError::Symbol {
                    input: s.to_string(),
                    message: "Failed to parse symbol".to_string(),
                })?;

                let value = Atom::num(*v);

                Ok((symbol, value))
            })
            .collect::<Result<HashMap<_, _>, SymbolicaError>>()?;

        Ok(Expressions {
            exprs,
            params,
            functions: functions.to_vec(),
            replacements,
        })
    }

    pub const fn default_constants() -> &'static [(&'static str, f64)] {
        &[
            ("pi", std::f64::consts::PI),
            ("e", std::f64::consts::E),
            ("phi", std::f64::consts::GOLDEN_RATIO),
        ]
    }

    pub fn set_constant(
        &mut self,
        symbol: impl AsRef<str>,
        value: f64,
    ) -> Result<(), SymbolicEvalError> {
        let symbol = try_symbol!(symbol.as_ref()).map_err(|s| SymbolicaError::Symbol {
            input: s.to_string(),
            message: "Failed to parse symbol".to_string(),
        })?;

        self.replacements.insert(symbol, Atom::num(value));

        Ok(())
    }

    pub fn evaluator(&self) -> Result<ExpressionEvaluator<f64>, SymbolicaError> {
        let replacements = self
            .replacements
            .iter()
            .map(|(s, v)| Replacement::new(*s, v.clone()))
            .collect::<Vec<_>>();

        let exprs = self
            .exprs
            .iter()
            .map(|e| e.replace_multiple(&replacements))
            .collect::<Vec<_>>();

        let mut ev = Atom::evaluator_multiple(&exprs, &self.params);

        for f in &self.functions {
            let body = f.body.replace_multiple(&replacements);
            ev = ev.add_function(f.name, f.args.clone(), body)?;
        }

        // TODO: Support alternate maps
        let ev = ev.build()?.map_coeff(&|c| c.re.to_f64());

        Ok(ev)
    }
}
