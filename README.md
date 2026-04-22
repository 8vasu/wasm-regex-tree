# wasm-regex-tree

WebAssembly visualizer for Rust regular expressions.

Parses Rust regular expressions using [regex-syntax](https://docs.rs/regex-syntax) and converts resulting Abstract Syntax Tree (syntactic) and High-level Intermediate Representation (semantic) to SVG.

To run it in your web browser, [click here](https://soumendraganguly.com/regex).

![coverImage](cover.png)

## Build and run

```sh
make deploy && make serve
```

Then open `http://localhost:8080/app/` in your web browser.

## Make targets

```
check    cargo fmt --check and cargo clippy
fmt      cargo fmt
clippy   cargo clippy --fix
build    cargo build
test     cargo test
clean    cargo clean
deploy   wasm-pack build, copy WASM and glue JS to app/
serve    python3 http.server on port 8080
```
