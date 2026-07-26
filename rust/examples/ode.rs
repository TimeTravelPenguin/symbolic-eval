use symbolic_eval::{
    expressions::Expressions,
    ode::{self, eval_ode},
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ode_str = ["2/3*x - 4/3*x*y", "x*y - y"];
    let symbs = ["t", "x", "y"];

    let exprs = Expressions::new(&ode_str, &symbs, &[], Expressions::default_constants())?;
    let ode_config = ode::OdeConfig {
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

    let result = eval_ode(exprs, ode_config)?;

    for (i, vals) in result.iter().enumerate() {
        println!(
            "Step {}: {}",
            i,
            symbs
                .iter()
                .zip(vals.iter())
                .map(|(s, v)| format!("{} = {:.4}", s, v))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    Ok(())
}
