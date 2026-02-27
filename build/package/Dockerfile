# Build stage
FROM rust:1.93.1-alpine3.23 AS builder

RUN apk add --no-cache openssl-dev pkgconfig

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release 2>/dev/null || true

# Copy source
COPY src ./src

# Build (touch main.rs to invalidate cache)
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.23

RUN apk --no-cache add ca-certificates openssl

WORKDIR /app
COPY --from=builder /app/target/release/example-rust-dapr-otel .

EXPOSE 8080

ENTRYPOINT ["./example-rust-dapr-otel"]
