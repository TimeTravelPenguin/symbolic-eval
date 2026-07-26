//! Symbolic expression evaluation for the `symbolic-eval` Typst plugin.
//!
//! This crate compiles to a WebAssembly plugin that a Typst document can call
//! to parse and numerically evaluate symbolic expressions using [`symbolica`].
//! It offers two capabilities:
//!
//! - [`evaluation`]: sampling one or more expressions over a Cartesian grid of
//!   parameter domains.
//! - [`ode`]: integrating a system of ODEs whose right-hand
//!   sides are given symbolically.
//!
//! The [`wasm`] module holds the byte-oriented entry points exposed to Typst;
//! the remaining modules make up the native Rust API (also usable directly, as
//! shown in `examples/`). Data crosses the WASM boundary as CBOR (see [`codec`]).

mod codec;
pub mod error;
pub mod evaluation;
pub mod expressions;
pub mod ode;
pub mod wasm;

pub use error::SymbolicEvalError;

use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::initiate_protocol;

#[cfg(target_arch = "wasm32")]
initiate_protocol!();

#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU64, Ordering};

/// Custom [`getrandom`] backend for the `wasm32-unknown-unknown` target.
///
/// [`symbolica`] pulls in [`getrandom`], but the bare WASM target has no OS
/// randomness source, so linking fails unless a backend is provided. This fills
/// the destination buffer with a cheap `xorshift`-derived stream seeded from a
/// process-wide atomic counter. It is **not** cryptographically secure and is
/// only intended to satisfy the symbol at link time; the plugin's actual work
/// (parsing and evaluation) is deterministic and does not depend on randomness.
///
/// See the crate `README` for background on why this is needed.
///
/// # Safety
///
/// `dest` must point to at least `len` writable, properly aligned bytes; the
/// buffer is written in full.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> Result<(), getrandom::Error> {
    static STATE: AtomicU64 = AtomicU64::new(0x4d59_5df4_d0f3_3173);

    let mut state = STATE
        .fetch_add(0x9e37_79b9_7f4a_7c15, Ordering::Relaxed)
        .wrapping_add(len as u64);
    let bytes = unsafe { std::slice::from_raw_parts_mut(dest, len) };

    for byte in bytes {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        *byte = state.wrapping_mul(0x2545_f491_4f6c_dd1d) as u8;
    }

    Ok(())
}

/// A user-defined function as received from Typst, before parsing.
///
/// The string fields are parsed into a [`Function`](crate::expressions::Function)
/// once they reach the native side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginArgsFunction {
    /// The function's name, e.g. `"f"`.
    pub name: String,
    /// The names of the function's formal parameters, e.g. `["x", "y"]`.
    pub args: Vec<String>,
    /// The function body as an expression string, e.g. `"x^2 + y^2"`.
    pub body: String,
}

/// The expression-evaluation payload sent from Typst, before parsing.
///
/// Decoded from CBOR at the WASM boundary and turned into an
/// [`Expressions`](crate::expressions::Expressions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginArgsExpressions {
    /// The expression strings to evaluate.
    pub exprs: Vec<String>,
    /// The names of the free parameters, in the order values will be supplied.
    pub params: Vec<String>,
    /// Any user-defined functions referenced by the expressions.
    pub functions: Vec<PluginArgsFunction>,
}

#[cfg(test)]
mod tests {
    //! Symbolica-backed integration checks.
    //!
    //! Unlicensed Symbolica enforces a single-instance-per-machine policy: it
    //! records the thread that first initializes it and calls `abort()` if it is
    //! ever touched from another thread (or another process). Because the test
    //! harness runs every `#[test]` on its own thread, *all* checks that parse or
    //! evaluate expressions must share one thread. They are therefore collected
    //! into the single [`symbolica_backed`] test below; each scenario is a plain
    //! helper `fn` so failures still point at a named case.

    use crate::error::SymbolicaError;
    use crate::evaluation::{EvaluationResult, eval_exprs};
    use crate::expressions::{Expressions, Function, SymbolDomain};
    use crate::ode::{self, OdeConfig};
    use crate::{PluginArgsExpressions, PluginArgsFunction, SymbolicEvalError};

    /// A typed empty constant slice for `Expressions::new`.
    const NO_CONSTANTS: &[(&str, f64)] = &[];

    fn expressions(exprs: &[&str], params: &[&str]) -> Expressions {
        Expressions::new(exprs, params, &[], NO_CONSTANTS).unwrap()
    }

    /// Evaluates `exprs` (parameters `params`) at the single point `inputs`.
    fn eval_at(exprs: &[&str], params: &[&str], inputs: &[f64]) -> Vec<f64> {
        let mut ev = expressions(exprs, params).evaluator().unwrap();

        let mut out = vec![0.0; exprs.len()];
        ev.evaluate(inputs, &mut out);

        out
    }

    fn parse_errors_are_reported() {
        let err = Expressions::new(&["x +"], &["x"], &[], NO_CONSTANTS).unwrap_err();

        assert!(matches!(
            err,
            SymbolicEvalError::SymbolicaError(SymbolicaError::Parse { .. })
        ));
    }

    fn polynomials_evaluate() {
        assert_eq!(eval_at(&["x^2 + 1"], &["x"], &[3.0]), [10.0]);
    }

    fn multiple_expressions_evaluate_at_once() {
        assert_eq!(
            eval_at(&["x + y", "x * y"], &["x", "y"], &[2.0, 5.0]),
            [7.0, 10.0]
        );
    }

    fn default_constants_are_substituted() {
        let out = eval_at(&["pi"], &[], &[]);

        assert!((out[0] - std::f64::consts::PI).abs() < 1e-12);
    }

    fn set_constant_overrides_a_symbol() {
        let mut exprs = expressions(&["k * x"], &["x"]);
        exprs.set_constant("k", 4.0).unwrap();

        let mut ev = exprs.evaluator().unwrap();
        let mut out = [0.0];
        ev.evaluate(&[3.0], &mut out);

        assert_eq!(out, [12.0]);
    }

    fn user_defined_functions_are_registered() {
        let f = Function::new("f", &["x", "y"], "x^2 + y^2").unwrap();
        let exprs = Expressions::new(&["f(x, y)"], &["x", "y"], &[f], NO_CONSTANTS).unwrap();

        let mut ev = exprs.evaluator().unwrap();
        let mut out = [0.0];
        ev.evaluate(&[3.0, 4.0], &mut out);

        assert_eq!(out, [25.0]);
    }

    fn function_new_reports_a_bad_body() {
        let err = Function::new("f", &["x"], "x +").unwrap_err();

        assert!(matches!(
            err,
            SymbolicEvalError::SymbolicaError(SymbolicaError::Parse { .. })
        ));
    }

    fn a_domain_is_sampled_with_exact_endpoints() {
        let domain = SymbolDomain {
            min: 0.0,
            max: 2.0,
            samples: 3,
        };

        let results = eval_exprs(expressions(&["x^2"], &["x"]), vec![domain]).unwrap();
        let xs: Vec<f64> = results.iter().map(|(inputs, _)| inputs[0]).collect();
        let ys: Vec<f64> = results.iter().map(|(_, out)| out[0]).collect();

        assert_eq!(xs, [0.0, 1.0, 2.0]);
        assert_eq!(ys, [0.0, 1.0, 4.0]);
    }

    fn a_single_sample_yields_the_minimum() {
        let domain = SymbolDomain {
            min: -3.0,
            max: 5.0,
            samples: 1,
        };

        let results = eval_exprs(expressions(&["x"], &["x"]), vec![domain]).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, [-3.0]);
    }

    fn domains_form_a_cartesian_product() {
        let domains = vec![
            SymbolDomain {
                min: 0.0,
                max: 1.0,
                samples: 2,
            },
            SymbolDomain {
                min: 0.0,
                max: 10.0,
                samples: 3,
            },
        ];

        let grid = eval_exprs(expressions(&["x + y"], &["x", "y"]), domains).unwrap();

        assert_eq!(grid.len(), 6);
        assert_eq!(grid[0], (vec![0.0, 0.0], vec![0.0]));
        assert_eq!(grid[1], (vec![0.0, 5.0], vec![5.0]));
        assert_eq!(grid.last().unwrap(), &(vec![1.0, 10.0], vec![11.0]));
    }

    fn rk4_config(t_span: (f64, f64), dt: f64, initial_conditions: Vec<f64>) -> OdeConfig {
        OdeConfig {
            method: peroxide::numerical::ode::RK4.into(),
            t_span,
            dt,
            initial_conditions,
        }
    }

    fn ode_integrates_a_constant_derivative_exactly() {
        // dy/dt = 1, y(0) = 0  =>  y(t) = t, so the state must always equal the
        // time. The fixed-step solver may overshoot the end of the interval by
        // up to one step, so we don't pin the final t exactly.
        let result = ode::solve_ode(
            expressions(&["1"], &["t", "y"]),
            rk4_config((0.0, 1.0), 0.1, vec![0.0]),
        )
        .unwrap();

        assert!(
            result.iter().all(|row| (row[1] - row[0]).abs() < 1e-9),
            "y should track t exactly: {result:?}"
        );

        let t_final = result.last().unwrap()[0];
        assert!((1.0..1.2).contains(&t_final), "final t was {t_final}");
    }

    fn ode_rows_are_t_then_state() {
        // Two state variables => each row is [t, y0, y1].
        let result = ode::solve_ode(
            expressions(&["1", "1"], &["t", "y0", "y1"]),
            rk4_config((0.0, 0.5), 0.1, vec![0.0, 0.0]),
        )
        .unwrap();

        assert!(result.iter().all(|row| row.len() == 3));
    }

    fn ode_approximates_exponential_growth() {
        // dy/dt = y, y(0) = 1  =>  y(t) = e^t. Compare against the analytic
        // solution at whatever final time the solver reached.
        let result = ode::solve_ode(
            expressions(&["y"], &["t", "y"]),
            rk4_config((0.0, 1.0), 0.05, vec![1.0]),
        )
        .unwrap();
        let last = result.last().unwrap();
        let (t_final, y_final) = (last[0], last[1]);

        assert!(
            (y_final - t_final.exp()).abs() < 1e-3,
            "y was {y_final}, expected e^{t_final} = {}",
            t_final.exp()
        );
    }

    fn ode_honours_the_configured_time_span() {
        // Regression: the config's t_span must be used. Before the fix the range
        // was hardcoded to (0, 10); with the config it stops just past t = 2.
        let result = ode::solve_ode(
            expressions(&["1"], &["t", "y"]),
            rk4_config((0.0, 2.0), 0.1, vec![0.0]),
        )
        .unwrap();
        let t_final = result.last().unwrap()[0];

        assert!((2.0..3.0).contains(&t_final), "final t was {t_final}");
    }

    fn wasm_eval_expr_roundtrips_through_cbor() {
        let args = crate::codec::encode(&PluginArgsExpressions {
            exprs: vec!["f(x)".to_string()],
            params: vec!["x".to_string()],
            functions: vec![PluginArgsFunction {
                name: "f".to_string(),
                args: vec!["z".to_string()],
                body: "z^2".to_string(),
            }],
        })
        .unwrap();

        let domains = crate::codec::encode(&vec![SymbolDomain {
            min: 0.0,
            max: 3.0,
            samples: 4,
        }])
        .unwrap();

        let out = crate::wasm::evaluation::eval_expr(&args, &domains, &[]).unwrap();
        let results: EvaluationResult = crate::codec::decode(&out).unwrap();
        let ys: Vec<f64> = results.iter().map(|(_, out)| out[0]).collect();

        assert_eq!(ys, [0.0, 1.0, 4.0, 9.0]);
    }

    /// Runs every Symbolica-backed scenario on this single test thread.
    ///
    /// See the module docs for why they cannot be separate `#[test]`s.
    #[test]
    fn symbolica_backed() {
        parse_errors_are_reported();
        polynomials_evaluate();
        multiple_expressions_evaluate_at_once();
        default_constants_are_substituted();
        set_constant_overrides_a_symbol();
        user_defined_functions_are_registered();
        function_new_reports_a_bad_body();

        a_domain_is_sampled_with_exact_endpoints();
        a_single_sample_yields_the_minimum();
        domains_form_a_cartesian_product();

        ode_integrates_a_constant_derivative_exactly();
        ode_rows_are_t_then_state();
        ode_approximates_exponential_growth();
        ode_honours_the_configured_time_span();

        wasm_eval_expr_roundtrips_through_cbor();
    }
}
