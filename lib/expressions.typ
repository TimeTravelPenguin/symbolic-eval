/// Sampling symbolic expressions over a grid of parameter values.

#import "plugin.typ": plugin-handle

/// Evaluates expressions over the Cartesian product of the given domains.
///
/// Every expression is evaluated at every point of the grid formed by
/// `domains`. There must be one domain per entry in `symbols`, in the same
/// order. The constants `pi`, `e`, and `phi` are always available.
/// Symbolica parses numeric imaginary literals such as `2+3i`.
///
/// The result is an array with one entry per grid point. Each entry is a pair
/// `(inputs, outputs)`, where `inputs` holds the parameter values at that point
/// and `outputs` holds the value of each expression there. Real-only calls
/// return floats; calls with complex domains or complex outputs return
/// dictionaries with `re` and `im` fields.
///
/// ```typ
/// #let res = eval_exprs(
///   ("x^2 + y", "sin(pi * x)"),
///   ("x", "y"),
///   (),
///   (domain(0, 1, samples: 3), domain(-1, 1, samples: 3)),
/// )
/// #res.first()  // ((0.0, -1.0), (-1.0, 0.0))
/// ```
///
/// - exprs (array): The expressions to evaluate, each a `str`.
/// - symbols (array): The free parameter names, each a `str`, in the order the
///   `domains` are given.
/// - functions (array): User functions referenced by the expressions, each
///   built with `func`. Pass `()` if there are none.
/// - constants (array|dictionary): Constant values available to the expressions.
///   Values may be real numbers or complex values built with `complex`.
/// - domains (array): One `domain` per entry in `symbols`.
/// -> array
#let eval_exprs(exprs, symbols, functions, constants, domains) = {
  let is-number(value) = type(value) in (int, float)
  let is-complex-value(value) = {
    if type(value) != dictionary {
      false
    } else {
      let keys = value.keys()

      "re" in keys and "im" in keys and is-number(value.re) and is-number(value.im)
    }
  }
  let normalize-constant(value) = {
    if is-number(value) {
      (re: float(value), im: 0.0)
    } else {
      (re: float(value.re), im: float(value.im))
    }
  }

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
  assert.eq(type(domains), array, message: "domains must be an array")

  assert(
    type(constants) in (array, dictionary),
    message: "constants must be an array of pairs or a dictionary",
  )

  if type(constants) == dictionary {
    constants = constants.pairs()
  }

  assert(
    constants.all(((_, val)) => is-number(val) or is-complex-value(val)),
    message: "constant values must be int, float, or complex values",
  )

  constants = constants.map(((const, val)) => (
    const,
    normalize-constant(val),
  ))

  let args = cbor.encode((
    exprs: exprs,
    params: symbols,
    functions: functions,
  ))
  let domains = cbor.encode(domains)
  let constants = cbor.encode(constants)

  let result = plugin-handle.eval_expr(args, domains, constants)

  cbor(result)
}
