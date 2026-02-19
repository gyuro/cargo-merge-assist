.PHONY: fmt lint test check

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-features --all-targets

check: fmt lint test
