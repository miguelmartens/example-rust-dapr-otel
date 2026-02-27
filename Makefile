.PHONY: build run test lint fmt prettier update deps dev clean

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

prettier:
	prettier --write .

update:
	cargo update

deps:
	cargo tree
	cargo outdated 2>/dev/null || true

dev: clean build run

clean:
	cargo clean
