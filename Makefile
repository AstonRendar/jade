.PHONY: install dev android-dev android-apk fmt lint test coverage check

ANDROID_HOME ?= $(HOME)/Library/Android/sdk
NDK_HOME ?= $(lastword $(sort $(wildcard $(ANDROID_HOME)/ndk/*)))
JAVA_HOME ?= /Applications/Android Studio.app/Contents/jbr/Contents/Home
NDK_BIN = $(NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64/bin
ANDROID_TARGETS = aarch64_linux_android armv7_linux_androideabi i686_linux_android x86_64_linux_android
ANDROID_TOOLS = $(foreach target,$(ANDROID_TARGETS),AR_$(target)="$(NDK_BIN)/llvm-ar" RANLIB_$(target)="$(NDK_BIN)/llvm-ranlib")
ANDROID_ENV = ANDROID_HOME="$(ANDROID_HOME)" NDK_HOME="$(NDK_HOME)" JAVA_HOME="$(JAVA_HOME)" $(ANDROID_TOOLS)

install:
	npm ci

dev:
	npm run tauri dev

android-dev:
	$(ANDROID_ENV) npm run tauri android dev

android-apk:
	$(ANDROID_ENV) npm run tauri android build -- --apk --debug --target aarch64

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
	cargo llvm-cov report --ignore-filename-regex 'src-tauri/src/(commands/|lib\.rs|main\.rs|scheduler\.rs|watcher\.rs)' --fail-under-lines 85
	cargo llvm-cov report --ignore-filename-regex '(/infrastructure/|/tests/|src-tauri/)' --fail-under-lines 95
	npm run coverage

check: lint test coverage
