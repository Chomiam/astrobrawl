# --- Build Stage ---
FROM rust:1.85-slim as builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY server ./server
COPY client ./client

RUN cargo build --package astrobrawl-server --release

# --- Runtime Stage ---
FROM debian:bookworm-slim

WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/astrobrawl-server /app/astrobrawl-server

ENV PORT=3000
ENV HOST=0.0.0.0
EXPOSE 3000

CMD ["/app/astrobrawl-server"]
