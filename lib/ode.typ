/// Integrating systems of ordinary differential equations with symbolic
/// right-hand sides.
///
/// An integration run is described by an integrator (built with one of the
/// method constructors such as `rkf45` or `rk4`) wrapped up with `ode_config`,
/// which is then handed to `solve_ode` together with the equations.

#import "plugin.typ": plugin-handle

/// Builds the configuration for a fixed-step integrator (one with no adaptive
/// step-size control). Used internally by `rk4`, `rk5`, `rals3`, and `rals4`.
///
/// - name (str): The integrator's method tag understood by the plugin.
/// -> dictionary
#let _fixed_method(name) = (method: name, config: none)

/// Builds the configuration for an embedded adaptive integrator. Used
/// internally by `bs23`, `rkf45`, `dp45`, `tsit45`, and `rkf78`, which differ
/// only in their defaults.
///
/// - name (str): The integrator's method tag understood by the plugin.
/// - tol (float): The per-step local error tolerance.
/// - safety_factor (float): The multiplier applied to the estimated optimal
///   step size (typically slightly below 1).
/// - min_step_size (float): The smallest step size the integrator may take.
/// - max_step_size (float): The largest step size the integrator may take.
/// - max_step_iter (int): The maximum number of attempts allowed to satisfy
///   `tol` within a single step.
/// -> dictionary
#let _adaptive_method(
  name,
  tol,
  safety_factor,
  min_step_size,
  max_step_size,
  max_step_iter,
) = {
  assert(tol > 0, message: "tol must be greater than 0")
  assert(max_step_iter > 0, message: "max_step_iter must be greater than 0")
  assert(
    min_step_size <= max_step_size,
    message: "min_step_size must not exceed max_step_size",
  )

  (
    method: name,
    config: (
      tol: float(tol),
      safety_factor: float(safety_factor),
      min_step_size: float(min_step_size),
      max_step_size: float(max_step_size),
      max_step_iter: max_step_iter,
    ),
  )
}

/// The classic fixed-step Runge-Kutta method of 4th order. Pass the result as
/// the `method` of `ode_config`.
/// -> dictionary
#let rk4() = _fixed_method("rk4")

/// A fixed-step Runge-Kutta method of 5th order. Pass the result as the
/// `method` of `ode_config`.
/// -> dictionary
#let rk5() = _fixed_method("rk5")

/// Ralston's fixed-step 3rd order method. Pass the result as the `method` of
/// `ode_config`.
/// -> dictionary
#let rals3() = _fixed_method("rals3")

/// Ralston's fixed-step 4th order method. Pass the result as the `method` of
/// `ode_config`.
/// -> dictionary
#let rals4() = _fixed_method("rals4")

/// The Bogacki-Shampine adaptive method (3rd order, 2nd order error estimate).
/// Pass the result as the `method` of `ode_config`.
///
/// - tol (float): The per-step local error tolerance.
/// - safety_factor (float): The step-size safety multiplier.
/// - min_step_size (float): The smallest permitted step size.
/// - max_step_size (float): The largest permitted step size.
/// - max_step_iter (int): The maximum step attempts per step.
/// -> dictionary
#let bs23(
  tol: 1e-3,
  safety_factor: 0.9,
  min_step_size: 1e-6,
  max_step_size: 1e-1,
  max_step_iter: 100,
) = _adaptive_method(
  "bs23",
  tol,
  safety_factor,
  min_step_size,
  max_step_size,
  max_step_iter,
)

/// The Runge-Kutta-Fehlberg adaptive method of 4th/5th order. Pass the result
/// as the `method` of `ode_config`.
///
/// - tol (float): The per-step local error tolerance.
/// - safety_factor (float): The step-size safety multiplier.
/// - min_step_size (float): The smallest permitted step size.
/// - max_step_size (float): The largest permitted step size.
/// - max_step_iter (int): The maximum step attempts per step.
/// -> dictionary
#let rkf45(
  tol: 1e-6,
  safety_factor: 0.9,
  min_step_size: 1e-6,
  max_step_size: 1e-1,
  max_step_iter: 100,
) = _adaptive_method(
  "rkf45",
  tol,
  safety_factor,
  min_step_size,
  max_step_size,
  max_step_iter,
)

/// The Dormand-Prince adaptive method of 4th/5th order. Pass the result as the
/// `method` of `ode_config`.
///
/// - tol (float): The per-step local error tolerance.
/// - safety_factor (float): The step-size safety multiplier.
/// - min_step_size (float): The smallest permitted step size.
/// - max_step_size (float): The largest permitted step size.
/// - max_step_iter (int): The maximum step attempts per step.
/// -> dictionary
#let dp45(
  tol: 1e-6,
  safety_factor: 0.9,
  min_step_size: 1e-6,
  max_step_size: 1e-1,
  max_step_iter: 100,
) = _adaptive_method(
  "dp45",
  tol,
  safety_factor,
  min_step_size,
  max_step_size,
  max_step_iter,
)

/// The Tsitouras adaptive method of 4th/5th order. Pass the result as the
/// `method` of `ode_config`.
///
/// - tol (float): The per-step local error tolerance.
/// - safety_factor (float): The step-size safety multiplier.
/// - min_step_size (float): The smallest permitted step size.
/// - max_step_size (float): The largest permitted step size.
/// - max_step_iter (int): The maximum step attempts per step.
/// -> dictionary
#let tsit45(
  tol: 1e-6,
  safety_factor: 0.9,
  min_step_size: 1e-6,
  max_step_size: 1e-1,
  max_step_iter: 100,
) = _adaptive_method(
  "tsit45",
  tol,
  safety_factor,
  min_step_size,
  max_step_size,
  max_step_iter,
)

/// The Runge-Kutta-Fehlberg adaptive method of 7th/8th order, suited to
/// high-accuracy runs. Pass the result as the `method` of `ode_config`.
///
/// - tol (float): The per-step local error tolerance.
/// - safety_factor (float): The step-size safety multiplier.
/// - min_step_size (float): The smallest permitted step size.
/// - max_step_size (float): The largest permitted step size.
/// - max_step_iter (int): The maximum step attempts per step.
/// -> dictionary
#let rkf78(
  tol: 1e-7,
  safety_factor: 0.9,
  min_step_size: 1e-10,
  max_step_size: 1e-1,
  max_step_iter: 100,
) = _adaptive_method(
  "rkf78",
  tol,
  safety_factor,
  min_step_size,
  max_step_size,
  max_step_iter,
)

/// The implicit Gauss-Legendre method of 4th order, suited to stiff systems.
/// Pass the result as the `method` of `ode_config`.
///
/// Being implicit, each step solves a nonlinear system with the chosen
/// `solver`.
///
/// - solver (str): The inner nonlinear solver, either `"FixedPoint"` or
///   `"Broyden"`.
/// - tol (float): The convergence tolerance for the inner solver.
/// - max_step_iter (int): The maximum inner-solver iterations per step.
/// -> dictionary
#let gl4(solver: "FixedPoint", tol: 1e-8, max_step_iter: 100) = {
  assert(
    solver in ("FixedPoint", "Broyden"),
    message: "solver must be \"FixedPoint\" or \"Broyden\"",
  )
  assert(tol > 0, message: "tol must be greater than 0")
  assert(max_step_iter > 0, message: "max_step_iter must be greater than 0")

  (
    method: "gl4",
    config: (
      solver: solver,
      tol: float(tol),
      max_step_iter: max_step_iter,
    ),
  )
}

/// Bundles everything needed to integrate a system, apart from the equations
/// themselves. Pass the result to `solve_ode`.
///
/// - method (dictionary): The integrator to use, built with one of the method
///   constructors such as `rkf45` or `rk4`.
/// - t_span (array): The integration interval as a pair `(t_start, t_end)`.
/// - dt (int, float): The initial step size. Adaptive methods refine it as they
///   go; fixed-step methods use it throughout.
/// - initial_conditions (array): The starting value of each state variable, as
///   `float` values, in the same order as the equations.
/// -> dictionary
#let ode_config(method, t_span, dt, initial_conditions) = {
  assert.eq(type(method), dictionary, message: "method must be a dictionary")
  assert(
    type(t_span) == array and t_span.len() == 2,
    message: "t_span must be a pair (t_start, t_end)",
  )
  assert(
    t_span.at(0) < t_span.at(1),
    message: "t_start must be less than t_end",
  )
  assert.eq(
    type(initial_conditions),
    array,
    message: "initial_conditions must be an array",
  )

  (
    method: method,
    t_span: (float(t_span.at(0)), float(t_span.at(1))),
    dt: float(dt),
    initial_conditions: initial_conditions.map(float),
  )
}

/// Integrates the system `dy/dt = f(t, y)` where each component of `f` is a
/// symbolic expression.
///
/// The expressions form the right-hand side, one per state variable. `symbols`
/// must be `t` followed by the state variables, matching the order of
/// `config.initial_conditions`.
///
/// The result is an array with one row per accepted step. Each row is
/// `(t, y_0, y_1, ...)`, so its length is one more than the number of state
/// variables.
///
/// ```typ
/// // Lotka-Volterra predator-prey model.
/// #let trajectory = solve_ode(
///   ("2/3*x - 4/3*x*y", "x*y - y"),
///   ("t", "x", "y"),
///   (),
///   ode_config(rkf45(), (0, 10), 0.1, (1.0, 1.0)),
/// )
/// #trajectory.first()  // (0.0, 1.0, 1.0)
/// ```
///
/// - exprs (array): The right-hand-side expressions, one per state variable,
///   each a `str`.
/// - symbols (array): `t` followed by the state-variable names, each a `str`.
/// - functions (array): User functions referenced by the expressions, each
///   built with `func`. Pass `()` if there are none.
/// - config (dictionary): The integrator configuration, built with
///   `ode_config`.
/// -> array
#let solve_ode(exprs, symbols, functions, constants, config) = {
  assert.eq(type(exprs), array, message: "exprs must be an array")
  assert(
    exprs.all(expr => type(expr) == str),
    message: "all exprs must be strings",
  )
  assert.eq(type(symbols), array, message: "symbols must be an array")
  assert(
    symbols.all(symbol => type(symbol) == str),
    message: "all symbols must be strings",
  )
  assert.eq(type(functions), array, message: "functions must be an array")
  assert.eq(type(config), dictionary, message: "config must be a dictionary")

  assert(
    type(constants) in (array, dictionary),
    message: "constants must be an array of (str, int|float) pairs or a dictionary",
  )

  if type(constants) == dictionary {
    constants = constants.pairs()
  }

  assert(
    constants.all(((_, val)) => type(val) in (int, float)),
    message: "constant values must be int or float",
  )

  constants = constants.map(((const, val)) => (
    const,
    float(val),
  ))

  let args = cbor.encode((
    exprs: exprs,
    params: symbols,
    functions: functions,
  ))
  let config = cbor.encode(config)
  let constants = cbor.encode(constants)

  let result = plugin-handle.solve_ode(args, config, constants)

  cbor(result)
}
