//! Integrate a symbolic ODE system: the Lotka-Volterra predator-prey model.
//!
//! The system is
//!
//! ```text
//! dx/dt = a x - b x y     (prey)
//! dy/dt = d x y - c y     (predators)
//! ```
//!
//! with parameters `a = 2/3`, `b = 4/3`, `c = 1`, and `d = 1`, and with
//! both populations starting at 1. Because the two populations feed back
//! on each other, the solution oscillates rather than settling. Run with:
//!
//! ```sh
//! cargo run --example ode
//! ```

use symbolic_eval::{
    expressions::Expressions,
    ode::{self, solve_ode},
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // One RHS expression per state variable. They may reference `t` and any of
    // the state variables; here only `x` and `y` appear.
    let exprs = ["a*x - b*x*y", "d*x*y - c*y"];
    let coeffs = [("a", 2.0 / 3.0), ("b", 4.0 / 3.0), ("c", 1.0), ("d", 1.0)];

    // The parameters are `t` followed by the state variables, matching the input
    // layout the solver feeds to the evaluator each step.
    let params = ["t", "x", "y"];

    let expressions = Expressions::new(&exprs, &params, &[], &coeffs)?;

    // Integrate from t = 0 to t = 10 with the adaptive RKF45 method, starting
    // from x = y = 1. `dt` is the initial step size; RKF45 adapts it to keep the
    // local error under `tol`.
    let config = ode::OdeConfig {
        method: peroxide::numerical::ode::RKF45 {
            tol: 1e-6,
            min_step_size: 1e-6,
            max_step_size: 0.1,
            max_step_iter: 1000,
            safety_factor: 0.9,
        }
        .into(),
        t_span: (0.0, 10.0),
        dt: 0.1,
        initial_conditions: vec![1.0, 1.0],
    };

    let trajectory = solve_ode(expressions, config)?;

    // Each row is `[t, x, y]`, lining up one-to-one with `params`.
    let header: String = params.iter().map(|name| format!("{name:>10}")).collect();
    println!("{header}");

    for row in &trajectory {
        let line: String = row.iter().map(|value| format!("{value:>10.4}")).collect();
        println!("{line}");
    }

    Ok(())
}
