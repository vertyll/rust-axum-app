# Base stage with common dependencies
FROM rust:1.85.0-slim AS base
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
COPY Cargo.toml Cargo.lock ./
# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {println!(\"Dummy build\");}" > src/main.rs && \
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
# Copy any necessary resources
COPY --from=builder /app/resources ./resources

CMD ["./rust-axum-app"]