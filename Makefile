.PHONY: check fmt clippy build test clean serve

check:
	cargo fmt -- --check && cargo clippy

fmt:
	cargo fmt

clippy:
	cargo clippy --fix

build:
	wasm-pack build --target web
	cp pkg/wasm_regex_tree.js app/
	cp pkg/wasm_regex_tree_bg.wasm app/

test:
	cargo test

clean:
	cargo clean
	rm -f app/wasm_regex_tree.js app/wasm_regex_tree_bg.wasm

serve:
	cargo run --bin serve -- 8080
