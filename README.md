# Hortulus

Watercolour flora and gilt ornaments grown in the margins of Typst documents.

This is the development repository. The package's documentation is in
[`package/README.md`](package/README.md).

- `package/`: the Typst package, as published.
- `plugin/`: the Rust source of the package's WebAssembly plugin.
- `reference/`: the Lua generator of [tikz-flora](https://github.com/wn-mitch/tikz-flora) (MIT),
  which the plugin ports and is checked against.
- `tools/`: the build script and the tests.

## Building the plugin

You need a Rust toolchain with the `wasm32-unknown-unknown` target, and `wasm-opt` from Binaryen.

```sh
RUSTUP_HOME=... CARGO_HOME=... tools/build_wasm.sh   # see the script for the paths it sets
cp build/hortulus.wasm package/hortulus.wasm
```

## Checking against the reference

With `texlua` on your path and the native build in place (`cargo build --release` in `plugin/`),
`tools/run_matrix.sh` paints each case with both generators and compares the output byte for byte.

## License

MIT. See `LICENSE`. `reference/` keeps its own license.
