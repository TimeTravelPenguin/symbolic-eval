#import "lib/lib.typ": *

#let res = eval_expr("x^2 + 2*x + 1", "x", domain(-10, 10, samples: 100))
#res.first() \
...#(res.len() - 2) more...\
#res.last()
