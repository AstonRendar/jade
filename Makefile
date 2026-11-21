.PHONY: fmt lint test coverage check

fmt:
	cargo fmt --all

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

coverage:
	cargo llvm-cov --workspace --no-report
	cargo llvm-cov report --fail-under-lines 85
	cargo llvm-cov report --ignore-filename-regex '(/infrastructure/|/tests/)' --fail-under-lines 95

check: lint test
