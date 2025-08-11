FROM rust:1.82 as builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /app/target/release/solana_usdc_indexer /usr/local/bin/

EXPOSE 8000

CMD ["solana_usdc_indexer"]
