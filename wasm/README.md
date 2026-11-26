# sky.wasm

The per-pixel nebula loop of `buildSpace()` in `index.html`, written in Rust and compiled to a ~3.7 KB WebAssembly module.
The module is embedded in `index.html` as base64 (`SKYW`). If WebAssembly can't load, the JavaScript loop next to it does the same job.

Rebuild (the build needs the Rust source in the toolchain, so `rust-src` must be installed):

    RUSTC_BOOTSTRAP=1 cargo build --release --target wasm32-unknown-unknown -Zbuild-std=core

then base64 `target/wasm32-unknown-unknown/release/sky.wasm` into the `atob('…')` in `index.html`.
