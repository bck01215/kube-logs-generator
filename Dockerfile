FROM rust as rust-builder
WORKDIR /app
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY src/ src
RUN cargo build --release


COPY --from=rust-builder /app/target/release/kube-logs-generator /app/kube-logs-generator
ENTRYPOINT  ["/app/kube-logs-generator"]
