.PHONY: check fmt clippy build test deploy

check:
	cargo fmt -- --check && cargo clippy

fmt:
	cargo fmt

clippy:
	cargo clippy --fix

build:
	cargo build

test:
	cargo test

clean:
	cargo clean

deploy:
	wasm-pack build --target web
	cp pkg/wasm_regex_tree.js app/
	cp pkg/wasm_regex_tree_bg.wasm app/

serve:
	python3 -m http.server 8080
