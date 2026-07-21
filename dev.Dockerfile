# Base stage with common dependencies
# Toasty 0.8 requires Rust 1.95+ (edition 2024).
FROM rust:1.95-slim AS base
RUN apt-get update && apt-get install -y \
    libpq-dev \
    build-essential \
    pkg-config

# Development stage
FROM base AS development
# Install cargo-watch for hot reloading
RUN cargo install cargo-watch

WORKDIR /app

# Set environment variables for better debugging
ENV RUST_BACKTRACE=1

# Command to run with hot reloading
CMD ["cargo", "watch", "-q", "-c", "-w", "src/", "-x", "run"]

# Production build stage
FROM base AS builder
WORKDIR /app
# Copy manifests
COPY Cargo.toml ./
# Create dummy binaries to pre-build dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {println!(\"Dummy build\");}" > src/main.rs && \
    echo "fn main() {}" > src/bin/cli.rs && \
    echo "" > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY . .
# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/rust-axum-app .
# Copy runtime resources (e-mail templates, translations, migration config)
COPY --from=builder /app/resources ./resources
COPY --from=builder /app/translations ./translations
COPY --from=builder /app/Toasty.toml ./Toasty.toml

CMD ["./rust-axum-app"]
