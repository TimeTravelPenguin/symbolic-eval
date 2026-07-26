//! Parsing symbolic input and compiling it into a reusable evaluator.
//!
//! [`Expressions`] is the central type: it bundles the parsed expressions,
//! their free parameters, any user-defined functions, and a table of constant
//! replacements. Calling [`Expressions::evaluator`] folds the constants in and
//! produces a [`symbolica`] [`ExpressionEvaluator`] that can be evaluated many
//! times cheaply.

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use symbolica::prelude::*;

use crate::{SymbolicEvalError, error::SymbolicaError};

/// Parses each string in `exprs` into a [`symbolica`] [`Atom`], collecting into
/// any [`FromIterator<Atom>`] container `C`.
///
/// # Errors
///
/// Returns [`SymbolicaError::Parse`] (wrapped in [`SymbolicEvalError`]) on the
/// first string that fails to parse.
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

/// Parses each string in `symbols` into a [`symbolica`] [`Symbol`], collecting
/// into any [`FromIterator<Symbol>`] container `C`.
///
/// # Errors
///
/// Returns [`SymbolicaError::Symbol`] (wrapped in [`SymbolicEvalError`]) on the
/// first string that is not a valid symbol.
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

/// A closed interval `[min, max]` sampled at `samples` evenly-spaced points.
///
/// Used by [`eval_exprs`](crate::evaluation::eval_exprs) to describe the range
/// of one parameter. Endpoints are always included exactly (see that function
/// for the sampling details).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolDomain {
    /// The lower bound of the interval (inclusive).
    pub min: f64,
    /// The upper bound of the interval (inclusive).
    pub max: f64,
    /// The number of sample points; `1` yields just `min`.
    pub samples: usize,
}

/// A user-defined function with its name, formal parameters, and parsed body.
///
/// This is the parsed counterpart of
/// [`PluginArgsFunction`](crate::PluginArgsFunction). Its `body` may reference
/// the symbols in `args`, which are substituted at evaluation time.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    /// The parsed function name.
    pub name: Symbol,
    /// The parsed formal parameters, in declaration order.
    pub args: Vec<Symbol>,
    /// The parsed function body.
    pub body: Atom,
}

impl Function {
    /// Parses a function's name, parameter names, and body from strings.
    ///
    /// # Errors
    ///
    /// Returns [`SymbolicaError::Symbol`] if the name or a parameter is not a
    /// valid symbol, or [`SymbolicaError::Parse`] if the body does not parse.
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

/// A parsed set of expressions ready to be compiled into an evaluator.
///
/// Groups the expressions to evaluate, the free `params` supplied at evaluation
/// time, any user-defined `functions`, and a `replacements` table of constants
/// (such as `pi`) that are substituted before compilation.
#[derive(Debug, Clone, PartialEq)]
pub struct Expressions {
    /// The parsed expressions to evaluate, one output per expression.
    pub exprs: Vec<Atom>,
    /// The free parameters, as parsed atoms, in the order values are supplied.
    pub params: Vec<Atom>,
    /// User-defined functions referenced by the expressions.
    pub functions: Vec<Function>,
    /// Symbol-to-value substitutions applied before compilation (constants).
    pub replacements: HashMap<Symbol, Atom>,
}

impl Expressions {
    /// Parses raw string input into an [`Expressions`].
    ///
    /// `constants` is a list of `(name, value)` pairs substituted for the named
    /// symbols before compilation; see [`default_constants`](Self::default_constants).
    ///
    /// # Errors
    ///
    /// Returns [`SymbolicEvalError::ArgumentError`] if `exprs` is empty, or a
    /// parse error if any expression, parameter, or constant name is invalid.
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

    /// The built-in mathematical constants made available to every expression:
    /// `pi`, `e`, and `phi` (the golden ratio).
    pub const fn default_constants() -> &'static [(&'static str, f64)] {
        &[
            ("pi", std::f64::consts::PI),
            ("e", std::f64::consts::E),
            ("phi", std::f64::consts::GOLDEN_RATIO),
        ]
    }

    /// Adds or overrides a constant substitution.
    ///
    /// The given `symbol` will be replaced by `value` in every expression (and
    /// function body) the next time [`evaluator`](Self::evaluator) is called.
    ///
    /// # Errors
    ///
    /// Returns [`SymbolicaError::Symbol`] if `symbol` is not a valid symbol.
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

    /// Compiles the expressions into a reusable [`ExpressionEvaluator<f64>`].
    ///
    /// Constants in [`replacements`](Self::replacements) are substituted into
    /// every expression and function body first; the user-defined functions are
    /// then registered, and finally the complex-valued coefficients produced by
    /// [`symbolica`] are projected onto their real parts (`c.re`).
    ///
    /// The returned evaluator takes one input per entry in
    /// [`params`](Self::params) and produces one output per expression.
    ///
    /// # Errors
    ///
    /// Returns a [`SymbolicaError`] if a function fails to register or the
    /// evaluator cannot be built.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A typed empty constant slice, so `Expressions::new` can infer its
    /// `impl AsRef<str>` type parameter when no constants are wanted.
    const NO_CONSTANTS: &[(&str, f64)] = &[];

    // NOTE: tests that actually parse or evaluate expressions are *not* here.
    // Unlicensed Symbolica aborts the process if it is called from more than one
    // thread, and the default test harness runs each `#[test]` on its own
    // thread. All Symbolica-backed checks therefore live in a single test in
    // `crate::tests`. The tests below only exercise logic that never touches
    // Symbolica.

    #[test]
    fn default_constants_are_the_expected_three() {
        let names: Vec<_> = Expressions::default_constants()
            .iter()
            .map(|(name, _)| *name)
            .collect();

        assert_eq!(names, ["pi", "e", "phi"]);
    }

    #[test]
    fn new_rejects_empty_expressions() {
        // The emptiness check happens before any parsing, so this stays off the
        // Symbolica code path.
        let err = Expressions::new(&[] as &[&str], &["x"], &[], NO_CONSTANTS).unwrap_err();

        assert!(matches!(err, SymbolicEvalError::ArgumentError(_)));
    }
}
