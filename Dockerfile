ARG RUST_IMAGE=rust:1.95-slim-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim

FROM ${RUST_IMAGE} AS deps
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin && echo "fn main() {}" > src/main.rs && echo "fn main() {}" > src/bin/cli.rs \
    && touch src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src

FROM deps AS build
COPY src/ src/
COPY translations/ translations/
COPY toasty/ toasty/
RUN touch src/main.rs src/lib.rs src/bin/cli.rs && cargo build --release --locked

FROM ${RUNTIME_IMAGE} AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 1001 app && useradd --system --uid 1001 --gid app app
WORKDIR /app
COPY --from=build /workspace/target/release/rust-axum-app ./
COPY resources/ resources/
COPY Toasty.toml ./
RUN mkdir -p uploads && chown app:app uploads

USER app
EXPOSE 3000

ENV APP_HOST=0.0.0.0 \
    APP_ENVIRONMENT=production

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
  CMD ["bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/3000 && printf 'GET /health HTTP/1.0\\r\\n\\r\\n' >&3 && grep -q '\"UP\"' <&3"]

ENTRYPOINT ["./rust-axum-app"]
