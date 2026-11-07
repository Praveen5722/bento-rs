.PHONY: build test fmt check
build:
	cargo build --release
test:
	cargo test
fmt:
	cargo fmt --all
check:
	cargo check
