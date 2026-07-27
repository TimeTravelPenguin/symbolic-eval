//! Sample a small system of expressions over a 2-D grid of parameter values.
//!
//! This mirrors what the Typst `eval_exprs` wrapper does, but drives the native
//! API directly. Run with:
//!
//! ```sh
//! cargo run --example eval_exprs
//! ```

use symbolic_eval::{
    evaluation::eval_exprs,
    expressions::{Expressions, Function, SymbolDomain},
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Two expressions to evaluate at every grid point. They share the free
    // parameters `x` and `y`, reference the built-in constant `pi`, and call a
    // user-defined function `f`.
    let exprs = ["f(x, y) + 1", "sin(pi * x) + y"];
    let params = ["x", "y"];

    let functions = [Function::new("f", &["a", "b"], "a^2 + b^2")?];

    let expressions = Expressions::new(
        &exprs,
        &params,
        &functions,
        Expressions::default_constants(),
    )?;

    // One domain per parameter, in the same order as `params`. The Cartesian
    // product of these is a 3 x 3 grid of nine points.
    let domains = vec![
        SymbolDomain::RealDomain {
            min: 0.0,
            max: 1.0,
            samples: 3,
        },
        SymbolDomain::RealDomain {
            min: -1.0,
            max: 1.0,
            samples: 3,
        },
    ];

    let results = eval_exprs(expressions, domains)?;

    println!(
        "{:>6} {:>6} | {:>12} {:>12}",
        "x", "y", &exprs[0], &exprs[1]
    );
    for (inputs, outputs) in &results {
        println!(
            "{:>6.2} {:>6.2} | {:>12.4} {:>12.4}",
            inputs[0], inputs[1], outputs[0], outputs[1]
        );
    }

    Ok(())
}
