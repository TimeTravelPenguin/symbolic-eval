#import "lib/lib.typ": *

#let exprs = (
  "x^2 + 2*y + 1",
  "sin(x) + cos(y)",
  "exp(x - y) - 1",
)

#let symbols = (
  "x",
  "y",
)

#let domains = (
  domain(-10, 10, samples: 100),
  domain(-5, 5, samples: 50),
)

#let res = eval_exprs(exprs, symbols, (), domains)
#res.first() \
...#(res.len() - 2) more...\
#res.last()
