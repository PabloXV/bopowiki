FROM rust:1.82 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY data ./data
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/bopo_wiki /app/
COPY data/tsi.csv /app/data/
EXPOSE 3000
CMD ["./bopo_wiki"]