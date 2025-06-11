# syntax=docker/dockerfile:1.4
FROM rust:1.84-slim AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() { println!(\"Building dependencies only...\"); }" > src/main.rs

RUN --mount=type=cache,id=cargo_registry_cache_s3sw,target=/usr/local/cargo/registry \
    cargo build --release

RUN rm src/main.rs /app/target/release/aws-secret-wrapper /app/target/release/aws-secret-wrapper.d
    
RUN echo "aws_access_key: \"xxxxx\"" > config.yaml && \
    echo "aws_secret_key: \"xxxxx\"" >> config.yaml && \
    echo "aws_region: \"xxxxx\"" >> config.yaml

# Now copy the actual application source code.
COPY src ./src
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

RUN --mount=type=cache,id=cargo_registry_cache_s3sw,target=/usr/local/cargo/registry \
    cargo build --release
RUN rm /app/target/release/aws-secret-wrapper /app/target/release/aws-secret-wrapper.d

# Set the entrypoint for the container.
ENTRYPOINT ["/app/entrypoint.sh"]