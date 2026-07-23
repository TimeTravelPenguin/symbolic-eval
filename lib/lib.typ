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

#let eval_expr(expr, symbol, domain) = {
  let expr = cbor.encode(expr)
  let symbol = cbor.encode(symbol)
  let domain = cbor.encode(domain)

  let result = _plugin.eval_expr(expr, symbol, domain)

  cbor(result)
}
