/// Parameter domains for gridded expression evaluation.

/// Describes a complex scalar.
///
/// Use this for complex constants passed to `eval_exprs`.
///
/// - re (int, float): The real part.
/// - im (int, float): The imaginary part.
/// -> dictionary
#let complex(re, im) = {
  assert(
    type(re) in (int, float),
    message: "re must be an int or float",
  )
  assert(
    type(im) in (int, float),
    message: "im must be an int or float",
  )

  (
    re: float(re),
    im: float(im),
  )
}

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

/// Describes a rectangular complex domain sampled on an evenly-spaced grid.
///
/// The imaginary axis is sampled outermost and the real axis innermost, so the
/// real part varies fastest in the returned grid.
///
/// - min-re (int, float): The lower bound of the real part.
/// - max-re (int, float): The upper bound of the real part.
/// - min-im (int, float): The lower bound of the imaginary part.
/// - max-im (int, float): The upper bound of the imaginary part.
/// - samples-re (int): The number of sample points along the real axis.
/// - samples-im (int): The number of sample points along the imaginary axis.
/// -> dictionary
#let complex_domain(
  min-re,
  max-re,
  min-im,
  max-im,
  samples-re: 200,
  samples-im: 200,
) = {
  assert(min-re <= max-re, message: "min-re must not exceed max-re")
  assert(min-im <= max-im, message: "min-im must not exceed max-im")
  assert(samples-re > 0, message: "samples-re must be greater than 0")
  assert(samples-im > 0, message: "samples-im must be greater than 0")

  (
    min_re: float(min-re),
    max_re: float(max-re),
    min_im: float(min-im),
    max_im: float(max-im),
    samples_re: samples-re,
    samples_im: samples-im,
  )
}
