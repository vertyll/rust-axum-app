ARG RUST_IMAGE=rust:1.99-slim-trixie
ARG RUNTIME_IMAGE=debian:trixie-slim

FROM ${RUST_IMAGE} AS tools
RUN cargo install topcoat-cli --version 0.11.0 --locked

FROM tools AS build
WORKDIR /workspace
COPY Cargo.toml Cargo.lock build.rs styles.css components.toml ./
COPY src/ src/
RUN cargo build --release --locked && topcoat asset bundle --release

FROM ${RUNTIME_IMAGE} AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 1001 app && useradd --system --uid 1001 --gid app app
WORKDIR /app
COPY --from=build /workspace/target/release/rust-axum-app-frontend ./
COPY --from=build /workspace/target/release/assets/ assets/

USER app
EXPOSE 3000

ENV HOST=0.0.0.0 \
    PORT=3000 \
    APP_ENVIRONMENT=production

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
  CMD ["bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/3000 && printf 'GET /health HTTP/1.0\\r\\n\\r\\n' >&3 && grep -q 'UP' <&3"]

ENTRYPOINT ["./rust-axum-app-frontend"]
