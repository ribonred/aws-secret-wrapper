FROM rust:1.84-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release && \
    cp target/release/aws-secret-wrapper /usr/local/bin/

FROM debian:bullseye-slim
COPY --from=builder /usr/local/bin/aws-secret-wrapper /usr/local/bin/
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]