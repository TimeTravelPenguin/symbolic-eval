use std::cell::RefCell;

use peroxide::{numerical::ode, prelude::*};
use serde::Deserialize;
use symbolica::prelude::*;
use thiserror::Error;

use crate::{SymbolicEvalError, expressions::Expressions};

#[derive(Debug, Error)]
pub enum OdeError {
    #[error("ODE solver error: {0}")]
    OdeSolverError(String),
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct OdeConfig {
    pub method: OdeMethod,
    pub t_span: (f64, f64),
    pub dt: f64,
    pub initial_conditions: Vec<f64>,
}

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

struct SymbolicOde {
    exprs: RefCell<ExpressionEvaluator<f64>>,
}

impl SymbolicOde {
    fn new(exprs: ExpressionEvaluator<f64>) -> Self {
        Self {
            exprs: RefCell::new(exprs),
        }
    }
}

impl ODEProblem for SymbolicOde {
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
