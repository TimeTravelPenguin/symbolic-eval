#import "../lib/lib.typ": *

#set page(height: auto, width: auto, margin: 1cm)

#let exprs = (
  "x^2 + 2*y + 1",
  "sin(x * pi) + cos(y - e)",
  "exp(x - y) - 1",
  "f(x, y) + g(x)",
)

#let symbols = (
  "x",
  "y",
)

#let constants = (
  pi: calc.pi,
  e: calc.e,
)

#let functions = (
  func("f", ("x", "y"), "x^2 + y^2"),
  func("g", ("z",), "z * sin(z)"),
)

#let domains = (
  domain(-10, 10, samples: 100),
  domain(-5, 5, samples: 50),
)

#let res = eval_exprs(exprs, symbols, functions, constants, domains)
#res.first() \
...#(res.len() - 2) more...\
#res.last()
