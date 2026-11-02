#!/bin/sh
# Builds the plugin into build/hortulus.wasm. Needs rustup with wasm32-unknown-unknown, and binaryen.
set -e
export RUSTUP_HOME=$PWD/build/rustup CARGO_HOME=$PWD/build/cargo
(cd plugin && rustup run stable cargo build --release --lib --target wasm32-unknown-unknown --target-dir ../build/wasm 2>&1 | grep -E '^error|^warning: unused' || true)
wasm-opt -O3 --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-multivalue --enable-reference-types \
  build/wasm/wasm32-unknown-unknown/release/hortulus.wasm -o build/hortulus.wasm
ls -la build/hortulus.wasm | awk '{print "wasm", $5, "bytes"}'
