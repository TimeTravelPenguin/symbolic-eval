/// Symbolic expression evaluation for Typst, powered by
/// #link("https://symbolica.io")[Symbolica].
///
/// This is the package entry point; it re-exports the public API from the
/// focused modules that make it up:
///
/// - `domain`, `complex`, and `complex_domain` (from `domain.typ`) and `func`
///   (from `functions.typ`) build the inputs shared by both features.
/// - `eval_exprs` (from `expressions.typ`) samples one or more expressions over
///   a grid of parameter values.
/// - `solve_ode` (from `ode.typ`), together with `ode_config` and the method
///   constructors (`rk4`, `rkf45`, `gl4`, ...), integrates a system of ordinary
///   differential equations whose right-hand sides are given symbolically.
///
/// Expressions are plain strings such as `"x^2 + sin(pi * y)"`. The constants
/// `pi`, `e`, and `phi` (the golden ratio) are always available, and callers
/// may supply their own functions with `func`.

#import "domain.typ": complex, complex_domain, domain
#import "functions.typ": func
#import "expressions.typ": eval_exprs
#import "ode.typ": (
  bs23, dp45, gl4, ode_config, rals3, rals4, rk4, rk5, rkf45, rkf78, solve_ode,
  tsit45,
)
