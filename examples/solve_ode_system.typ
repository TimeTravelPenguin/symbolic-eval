#import "@preview/cetz:0.5.2": canvas, draw
#import "@preview/cetz-plot:0.1.4": plot
#import "../lib/lib.typ": *

#set page(height: auto, width: auto, margin: 1cm)

#let odes = (
  "a*x - b*x*y",
  "-c*y + d*x*y",
)
#let coeffs = (
  ("a", 2 / 3000),
  ("b", 4 / 3000),
  ("c", 1 / 1000),
  ("d", 1 / 1000),
)
#let symbs = ("t", "x", "y")
#let config = ode_config(rk4(), (0, 10), 0.01, (1.5, 1.5))
#let res = solve_ode(odes, symbs, (), coeffs, config)

#let f1 = res.map(((t, x, y)) => (t, x))
#let f2 = res.map(((t, x, y)) => (t, y))
#let f3 = res.map(((t, x, y)) => (x, y))

#canvas({
  import draw: *

  set-style(
    axes: (stroke: .5pt, tick: (stroke: .5pt)),
    legend: (stroke: none, orientation: ttb, item: (spacing: .3), scale: 80%),
  )

  plot.plot(
    size: (12, 8),
    y-min: 0,
    y-max: 2.8,
    x-label: [Time ($t$)],
    y-label: [Population],
    legend: "east",
    {
      plot.add(
        f1,
        label: $ x(t) $,
        style: (stroke: blue),
      )

      plot.add(
        f2,
        label: $ y(t) $,
        style: (stroke: red),
      )
    },
  )
})

#v(5mm)

#canvas({
  import draw: *

  set-style(
    axes: (stroke: .5pt, tick: (stroke: .5pt)),
    legend: (stroke: none, orientation: ttb, item: (spacing: .3), scale: 80%),
  )

  plot.plot(
    size: (12, 8),
    y-min: 0,
    y-max: 1.7,
    x-min: 0,
    x-max: 2.9,
    x-label: [Pray Population ($x$)],
    y-label: [Predator Population ($y$)],
    legend: "east",
    {
      plot.add(
        f3,
        label: $ (x(t), y(t)) $,
        style: (stroke: red),
      )
    },
  )
})

