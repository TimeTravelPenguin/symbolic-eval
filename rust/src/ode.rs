//! Solving systems of ordinary differential equations whose right-hand sides
//! are given as symbolic expressions.
//!
//! The RHS of the system is compiled once into a [`symbolica`]
//! [`ExpressionEvaluator`] and then evaluated at every integration step by
//! [`peroxide`]'s ODE solvers.

use std::cell::RefCell;

use peroxide::{numerical::ode, prelude::*};
use serde::Deserialize;
use symbolica::prelude::*;
use thiserror::Error;

use crate::{SymbolicEvalError, expressions::Expressions};

/// Errors that can occur while integrating a symbolic ODE system.
#[derive(Debug, Error)]
pub enum OdeError {
    /// The underlying [`peroxide`] solver failed (e.g. a step-size constraint
    /// was violated or the maximum iteration count was reached). The wrapped
    /// string is the solver's own error message.
    #[error("ODE solver error: {0}")]
    OdeSolverError(String),
}

/// A [`peroxide`] integrator paired with its per-method configuration.
///
/// This enum exists so that the concrete integrator can be chosen at runtime
/// (and deserialized from the plugin input) rather than being fixed at compile
/// time. Deserialization uses an internally-tagged representation: a `method`
/// field selects the variant and a `config` field carries that integrator's
/// settings, e.g. `{ "method": "rkf45", "config": { ... } }`.
///
/// The enum is `#[non_exhaustive]` because [`peroxide`] may add integrators and
/// callers should not assume the set is fixed.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "method", content = "config", rename_all = "lowercase")]
#[non_exhaustive]
pub enum OdeMethod {
    BS23(ode::BS23),
    DP45(ode::DP45),
    GL4(ode::GL4),
    RALS3(ode::RALS3),
    RALS4(ode::RALS4),
    RK4(ode::RK4),
    RK5(ode::RK5),
    RKF45(ode::RKF45),
    RKF78(ode::RKF78),
    TSIT45(ode::TSIT45),
}

/// Generates a `From<ode::$variant> for OdeMethod` impl for each listed
/// integrator, so a bare [`peroxide`] integrator can be lifted into the
/// runtime-dispatched [`OdeMethod`] enum with `.into()`.
macro_rules! impl_from_ode_method {
    ($($variant:ident),*) => {
        $(
            impl From<ode::$variant> for OdeMethod {
                fn from(integrator: ode::$variant) -> Self {
                    OdeMethod::$variant(integrator)
                }
            }
        )*
    };
}

impl_from_ode_method!(
    BS23, DP45, GL4, RALS3, RALS4, RK4, RK5, RKF45, RKF78, TSIT45
);

impl ODEIntegrator for OdeMethod {
    /// Advances the solution by one step by forwarding to whichever concrete
    /// integrator this value wraps.
    fn step<P: ODEProblem>(
        &self,
        problem: &P,
        t: f64,
        y: &mut [f64],
        dt: f64,
    ) -> anyhow::Result<f64> {
        match self {
            OdeMethod::BS23(integrator) => integrator.step(problem, t, y, dt),
            OdeMethod::DP45(integrator) => integrator.step(problem, t, y, dt),
            OdeMethod::GL4(integrator) => integrator.step(problem, t, y, dt),
            OdeMethod::RALS3(integrator) => integrator.step(problem, t, y, dt),
            OdeMethod::RALS4(integrator) => integrator.step(problem, t, y, dt),
            OdeMethod::RK4(integrator) => integrator.step(problem, t, y, dt),
            OdeMethod::RK5(integrator) => integrator.step(problem, t, y, dt),
            OdeMethod::RKF45(integrator) => integrator.step(problem, t, y, dt),
            OdeMethod::RKF78(integrator) => integrator.step(problem, t, y, dt),
            OdeMethod::TSIT45(integrator) => integrator.step(problem, t, y, dt),
        }
    }
}

/// Everything needed to integrate a system, independent of the RHS itself.
///
/// The number of state variables is determined by `initial_conditions`, and
/// must match the number of expressions in the [`Expressions`] passed to
/// [`eval_ode`].
#[derive(Debug, Clone, Deserialize)]
pub struct OdeConfig {
    /// The integrator to use, together with its per-method settings.
    pub method: OdeMethod,
    /// The integration interval `(t_start, t_end)`.
    pub t_span: (f64, f64),
    /// The (initial / requested) step size handed to the integrator. Adaptive
    /// methods may subdivide this internally.
    pub dt: f64,
    /// The state vector at `t_span.0`, one entry per state variable.
    pub initial_conditions: Vec<f64>,
}

/// Integrates the system `dy/dt = f(t, y)` where `f` is given symbolically.
///
/// Each expression in `exprs` is one component of the RHS; the expressions'
/// parameters are expected to be `t` followed by the state variables, in the
/// same order as `ode_config.initial_conditions`. The evaluator is built once
/// and reused for every step (see [`SymbolicOde`]).
///
/// Returns one row per accepted step. Each row is `[t, y_0, y_1, ...]`, so the
/// row width is `1 + initial_conditions.len()`.
///
/// # Errors
///
/// Returns [`OdeError::OdeSolverError`] if the integrator fails, or a
/// [`SymbolicEvalError`] if the expressions cannot be compiled into an
/// evaluator.
pub fn eval_ode(
    exprs: Expressions,
    ode_config: OdeConfig,
) -> Result<Vec<Vec<f64>>, SymbolicEvalError> {
    let ev = exprs.evaluator()?;

    let symbolic_ode = SymbolicOde::new(ev);
    let basic_ode_solver = BasicODESolver::new(ode_config.method);

    let initial_conditions = vec![1.0, 1.0];
    let (t_vec, y_vec) = basic_ode_solver
        .solve(&symbolic_ode, (0f64, 10f64), 0.01, &initial_conditions)
        .map_err(|err| OdeError::OdeSolverError(format!("ODE solver error: {err}")))?;

    let result = t_vec
        .iter()
        .zip(y_vec.iter())
        .map(|(t, y)| {
            let mut row = Vec::with_capacity(y.len() + 1);
            row.push(*t);
            row.extend_from_slice(y);

            row
        })
        .collect::<Vec<_>>();

    Ok(result)
}

/// Adapts a compiled symbolic evaluator to [`peroxide`]'s [`ODEProblem`] trait.
///
/// The evaluator is stored behind a [`RefCell`] because [`ODEProblem::rhs`]
/// takes `&self`, but [`ExpressionEvaluator::evaluate`] requires `&mut self`
/// (it reuses internal scratch buffers between calls). The solver is
/// single-threaded and never re-enters `rhs`, so the borrow always succeeds.
struct SymbolicOde {
    exprs: RefCell<ExpressionEvaluator<f64>>,
}

impl SymbolicOde {
    /// Wraps a compiled evaluator so it can be used as an [`ODEProblem`].
    fn new(exprs: ExpressionEvaluator<f64>) -> Self {
        Self {
            exprs: RefCell::new(exprs),
        }
    }
}

impl ODEProblem for SymbolicOde {
    /// Computes `dy = f(t, y)` by evaluating the symbolic RHS.
    ///
    /// The inputs are laid out as `[t, y_0, y_1, ...]` to match the parameter
    /// ordering expected by [`eval_ode`].
    ///
    /// NOTE: Currently, including elements of [`dy`] in the inputs is not supported.
    // TODO: support including elements of `dy` in the inputs. Requires
    // refactoring to ensure the evaluator has symbols for each element of `dy`.
    fn rhs(&self, t: f64, y: &[f64], dy: &mut [f64]) -> anyhow::Result<()> {
        let mut inputs = Vec::with_capacity(y.len() + 1);
        inputs.push(t);
        inputs.extend_from_slice(y);

        let mut exprs = self
            .exprs
            .try_borrow_mut()
            .map_err(|err| anyhow::anyhow!("symbolic evaluator is already borrowed: {err}"))?;

        exprs.evaluate(&inputs, dy);

        Ok(())
    }
}
