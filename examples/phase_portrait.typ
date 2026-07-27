#import "../lib/lib.typ": *

#set page(height: auto, width: auto, margin: 6mm)

// Phase portrait of f(z) = (z^2 - 1 - i) / (z^2 + 1 + i).
//
// NOTE: This example is not at all practical, and is only meant to demonstrate the
// complex expression evaluator. It is very slow to render, and the results are not
// great. It would be best to implement this as a package feature utilising WASM,
// which is a planned feature.
//
// This uses the complex expression evaluator directly: `z` is sampled over a
// complex domain and each returned complex value is mapped to colour in Typst.
#let x-min = -calc.pi
#let x-max = calc.pi
#let y-min = -2.5
#let y-max = 2.5

#let x-samples = 360 // keep small for fast rendering but lower quality
#let y-samples = int(calc.round((y-max - y-min) / (x-max - x-min) * x-samples))
#let cell-size = 0.5pt

#let functions = (
  func("f", ("z",), "(z^2 - 1 - i) / (z^2 + 1 + i)"),
)

#let res = eval_exprs(
  ("f(z)",),
  ("z",),
  functions,
  (i: complex(0, 1)), // constant `i` for imaginary unit
  (
    complex_domain(
      x-min,
      x-max,
      y-min,
      y-max,
      samples-re: x-samples,
      samples-im: y-samples,
    ),
  ),
)

// Clamp a value between a minimum and maximum.
// Used to limit lightness in the phase colour mapping.
#let clamp(value, min, max) = calc.min(max, calc.max(min, value))

// Map a complex value to a colour based on its phase and magnitude.
// The phase is mapped to hue, and the magnitude is used to adjust lightness.
#let phase-colour(value) = {
  let magnitude = calc.sqrt(value.re * value.re + value.im * value.im)
  let log-mod = calc.ln(magnitude)
  let band = 0.5 + 0.5 * calc.cos(360deg * log-mod / calc.ln(2))
  let lightness = clamp(
    52% + 16% * calc.tanh(log-mod) + 7% * (band - 0.5),
    22%,
    78%,
  )

  color.hsl(calc.atan2(value.re, value.im), 92%, lightness)
}

// Create a small box representing a pixel with the colour
// corresponding to the complex value.
#let pixel(value) = box(
  width: cell-size,
  height: cell-size,
  fill: phase-colour(value),
)

// Create a grid of pixels representing the phase portrait.
// The grid is constructed by iterating over the sampled
// complex values and creating a pixel for each one.
#let pixels = ()
#for row in range(y-samples) {
  for col in range(x-samples) {
    let index = (y-samples - row - 1) * x-samples + col
    let (_, outputs) = res.at(index)

    pixels.push(pixel(outputs.first()))
  }
}

// Put the pixels together!
#grid(
  columns: (cell-size,) * x-samples,
  column-gutter: 0pt,
  row-gutter: 0pt,
  ..pixels,
)
