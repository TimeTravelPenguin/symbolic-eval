/// Loads the compiled Symbolica WebAssembly plugin.
///
/// All data crosses the plugin boundary as CBOR byte buffers. This handle is
/// shared by the modules that call into the plugin (`expressions` and `ode`);
/// end users never touch it directly.

/// The compiled Symbolica plugin handle.
#let plugin-handle = plugin("wasm/symbolic_eval.wasm")
