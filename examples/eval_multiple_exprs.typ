#import "../lib/lib.typ": *

#set page(height: auto, width: auto, margin: 1cm)

// The expressions we wish to evaluate.
#let exprs = (
  "x^2 + 2*y + 1",
  "sin(x * pi) + cos(y - e)",
  "exp(x - y) - 1",
  "f(x, y) + g(x)",
)

// The symbols used as input for the expressions.
#let symbols = (
  "x",
  "y",
)

// Additional optional constants.
#let constants = (
  pi: calc.pi,
  e: calc.e,
)

// Additional optional functions that may be used in expressions.
#let functions = (
  func("f", ("x", "y"), "x^2 + y^2"),
  func("g", ("z",), "z * sin(z)"),
)

// The domains over which to evaluate the expressions.
// The order of the domains must match the order of the symbols.
#let domains = (
  domain(-10, 10, samples: 4),
  domain(-5, 5, samples: 3),
)

$
  f(x, y) = vec(..#exprs.map(e => [#eval(e.replace("*", ""), mode: "math")]))
$
where
$
  #functions.map(f => eval(
    f.name + "(" + f.args.join(", ") + ") &= " + f.body.replace("*", ""),
    mode: "math",
  )).join($\ $)
$


#let res = eval_exprs(exprs, symbols, functions, constants, domains)

#show table: set align(right)
#show table.cell.where(y: 0): it => {
  set align(center)
  show math.equation: math.bold
  it
}

#table(
  columns: res.first().flatten().len(),
  table.header($x$, $y$, table.cell(
    align: center,
    colspan: exprs.len(),
    $f(x, y)$,
  )),
  ..res
    .map(row => row.flatten())
    .flatten()
    .map(v => [#calc.round(v, digits: 3)])
)
