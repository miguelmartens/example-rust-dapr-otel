.PHONY: build run test lint fmt tidy deps dev clean

BINARY := target/release/example-rust-dapr-otel

build:
	cargo build --release

run: build
	$(BINARY)

test:
	cargo test

lint:
	cargo clippy -- -D warnings
	cargo fmt --check

fmt:
	cargo fmt
	prettier --write "*.{json,md,yaml,yml}" 2>/dev/null || true

tidy:
	cargo update

deps:
	cargo tree
	cargo outdated 2>/dev/null || true

dev:
	cargo run

clean:
	cargo clean
