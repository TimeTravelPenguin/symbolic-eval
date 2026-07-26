/// Parameter domains for gridded expression evaluation.

/// Describes a closed interval `[min, max]` sampled at evenly-spaced points.
///
/// Pass one domain per free parameter to `eval_exprs`. The first and last
/// samples always land exactly on `min` and `max`.
///
/// - min (int, float): The lower bound of the interval.
/// - max (int, float): The upper bound of the interval. Must be greater than
///   `min`.
/// - samples (int): The number of sample points across the interval.
/// -> dictionary
#let domain(min, max, samples: 200) = {
  assert(min < max, message: "min must be less than max")
  assert(samples > 0, message: "samples must be greater than 0")

  (
    min: float(min),
    max: float(max),
    samples: samples,
  )
}
