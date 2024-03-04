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