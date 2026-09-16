# Create Rust App — root task entrypoints (mirrors sibling `make` wrappers)

.PHONY: test fmt fmt-check clippy build clean help

help:
	@echo "Targets: test fmt fmt-check clippy build clean"

test:
	cargo test --workspace --locked

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --locked -- -D warnings

build:
	cargo build --locked --bin create-rust-app

clean:
	cargo clean
	rm -rf dist
