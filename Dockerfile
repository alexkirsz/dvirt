FROM ekidd/rust-musl-builder:1.38.0 AS builder

WORKDIR /app

RUN sudo chown rust:rust .

USER rust

# Install and build dependencies first to avoid rebuilding on source change.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() { }" > src/main.rs && \
    cargo build --release && \
    rm -rf /app/target/x86_64-unknown-linux-musl/release/deps/dvirt*

# Build the program.
COPY . .
RUN cargo build --release

FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/dvirt /
EXPOSE 80

CMD ["/dvirt"]

