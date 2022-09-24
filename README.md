# Asteroids

Requires nightly Rust. [Play it in the browser](https://deadalusai.github.com/).

## Running

Run locally:
```
cargo run
```

Run in browser:
```
rustup target install wasm32-unknown-unknown
cargo install wasm-server-runner
cargo run --release --target wasm32-unknown-unknown
```

## Compiling for web

Run `wasm-bindgen` to generate all the files need to run in the browser.

```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/
```

## Reference

- https://bevy-cheatbook.github.io
- https://yqnn.github.io/svg-path-editor/
