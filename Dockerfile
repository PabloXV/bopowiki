FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/bopo_wiki /app/
COPY data/tsi.csv /app/data/
EXPOSE 3000
CMD ["./bopo_wiki"]