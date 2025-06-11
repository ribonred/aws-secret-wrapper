FROM rust:1.84-slim AS deps
# Note that we add wget here
RUN apt-get update && apt-get install --yes libpq-dev wget

# Install sccache to greatly speedup builds in the CI
RUN wget https://github.com/mozilla/sccache/releases/download/v0.10.0/sccache-v0.10.0-x86_64-unknown-linux-musl.tar.gz \
    && tar xzf sccache-v0.10.0-x86_64-unknown-linux-musl.tar.gz \
    && mv sccache-v0.10.0-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache \
    && chmod +x /usr/local/bin/sccache
WORKDIR /app
COPY dummy.rs .
COPY Cargo.toml .
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
COPY . .

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]