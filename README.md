# Rust Concurrency Demo with WebAssembly Threads

This project demonstrates using Rayon-based parallelism in WebAssembly using the wasm-bindgen-rayon crate.

## Features

- Parallel processing in the web browser via WebAssembly threads
- Uses SharedArrayBuffer and Web Workers for multi-threading
- Demonstrates BPE-based text processing with parallel algorithms

## Setup for Development

This project uses nightly Rust with WASM thread support. The following configuration is required:

1. Nightly Rust toolchain with specific features:
   ```
   rustup toolchain install nightly-2024-08-02
   rustup component add rust-src --toolchain nightly-2024-08-02
   ```

2. Cargo configuration (in `.cargo/config.toml`):
   ```toml
   [target.wasm32-unknown-unknown]
   rustflags = ["-C", "target-feature=+atomics,+bulk-memory", "--cfg=web_sys_unstable_apis"]

   [unstable]
   build-std = ["panic_abort", "std"]
   ```

3. Build using:
   ```
   rustup run nightly-2024-08-02 trunk build --release
   ```

## Cross-Origin Isolation

For SharedArrayBuffer to work in browsers, the page must be served with appropriate COOP/COEP headers:

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

These are added in the HTML file via meta tags.

## Implementation Notes

The WebAssembly thread pool is initialized at application startup before any Rayon operations are performed:

```rust
// Initialize the thread pool
let promise = egui_hex07::init_thread_pool(num_threads);
let result = wasm_bindgen_futures::JsFuture::from(promise).await;
```

This creates Web Workers for parallel computation, allowing Rayon's parallel iterators to work across threads. 