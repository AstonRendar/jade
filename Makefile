.PHONY: install dev fmt lint test coverage check

install:
	npm ci

dev:
	npm run tauri dev

fmt:
	cargo fmt --all
	npm run format

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	npm run lint
	npm run typecheck

test:
	cargo test --workspace
	npm test

coverage:
	cargo llvm-cov --workspace --no-report
	cargo llvm-cov report --fail-under-lines 85
	cargo llvm-cov report --ignore-filename-regex '(/infrastructure/|/tests/)' --fail-under-lines 95
	npm run coverage

check: lint test coverage
