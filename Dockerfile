FROM rust as rust-builder
WORKDIR /app
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY src/ src
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install --no-install-recommends -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/cache/apt

COPY --from=rust-builder /app/target/release/kube-logs-generator /app/kube-logs-generator
CMD  ["/app/kube-logs-generator"]
