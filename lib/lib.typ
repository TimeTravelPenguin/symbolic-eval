#let _plugin = plugin("wasm/symbolic_eval.wasm")

#let domain(min, max, samples: 200) = {
  assert(min < max, message: "min must be less than max")
  assert(samples > 0, message: "samples must be greater than 0")

  (
    min: float(min),
    max: float(max),
    samples: samples,
  )
}


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

#let eval_exprs(exprs, symbols, functions, domains) = {
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

  let input = cbor.encode((
    exprs: exprs,
    params: symbols,
    functions: functions,
    domains: domains,
  ))

  let result = _plugin.eval_expr(input)

  cbor(result)
}
