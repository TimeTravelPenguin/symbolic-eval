#import "@preview/cetz:0.5.2": canvas, draw
#import "@preview/cetz-plot:0.1.4": plot
#import "../lib/lib.typ": *

#set page(height: auto, width: auto, margin: 1cm)

// This is the Lotka-Volterra predator-prey system
// each line is one ODE in a system of ODEs:
// f(t, x, y) = (x'(t, x, y), y'(t, x, y))
#let odes = (
  "a*x - b*x*y",
  "-c*y + d*x*y",
)

// With constants added, they are very easy to use
#let coeffs = (
  ("a", 2 / 3000),
  ("b", 4 / 3000),
  ("c", 1 / 1000),
  ("d", 1 / 1000),
)

// Indep variable `t` and dep. variables `x` and `y` which
// make up the system
#let symbs = ("t", "x", "y")

// The config for the ODE solver, which requires:
// - The numerical method to use (here, RK4)
// - The time interval to solve over (here, from 0 to 10)
// - The time step to use (here, 0.01)
// - The initial conditions for the system (here, x(0) = 1.5, y(0) = 1.5).
//   Note that initial conditions may also refer to conditions on the derivatives,
//   e.g. x(0) = 1.5, x'(0) = 1.5, when solving higher-order ODEs, rather than systems
//   of first-order ODEs.
#let config = ode_config(rk4(), (0, 10), 0.01, (1.5, 1.5))

// The solution is returned as a list of tuples, where each tuple
// is (t, x(t), y(t)) for each time step.
#let res = solve_ode(odes, symbs, (), coeffs, config)

// Map the solution to three different lists of tuples, one for each plot we want to make:
// 1. The first plot is x(t) vs t, so we map to (t, x)
#let f1 = res.map(((t, x, y)) => (t, x))

// 2. The second plot is y(t) vs t, so we map to (t, y)
#let f2 = res.map(((t, x, y)) => (t, y))

// 3. The third plot is y(t) vs x(t), so we map to (x, y)
#let f3 = res.map(((t, x, y)) => (x, y))

// Plot the results!
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

