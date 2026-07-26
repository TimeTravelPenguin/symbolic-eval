/// User-defined functions that expressions may reference.

/// Defines a user function that expressions may reference.
///
/// For example, `func("f", ("x", "y"), "x^2 + y^2")` defines `f(x, y)` so that
/// an expression like `"f(a, b) + 1"` can call it.
///
/// - name (str): The function's name.
/// - args (array): The formal parameter names, each a `str`.
/// - body (str): The function body, written in terms of `args`.
/// -> dictionary
#let func(name, args, body) = {
  assert.eq(type(name), str, message: "name must be a string")
  assert.eq(type(args), array, message: "args must be an array")
  assert(
    args.all(arg => type(arg) == str),
    message: "all args must be strings",
  )
  assert.eq(type(body), str, message: "body must be a string")

  (
    name: name,
    args: args,
    body: body,
  )
}
