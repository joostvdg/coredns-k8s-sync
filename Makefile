.PHONY: build test clean

# Build the project in release mode
build:
	cargo build --release

# Run tests
test:
	cargo test

# Clean up build artifacts
clean:
	cargo clean

# Build a static binary
static:
	cargo rustc --release -- -C target-feature=+crt-static

static-musl:
	PKG_CONFIG_SYSROOT_DIR=/ cargo build --release --target x86_64-unknown-linux-musl

.PHONY: audit

audit:
	cargo audit

.PHONY: format lint

format:
	cargo fmt

lint:
	cargo clippy

all: format lint audit test
