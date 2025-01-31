FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin dial-a-poem

# Create debug info
RUN objcopy --only-keep-debug --compress-debug-sections=zlib /app/target/release/dial-a-poem /app/target/release/dial-a-poem.debug
RUN objcopy --strip-debug --strip-unneeded /app/target/release/dial-a-poem
RUN objcopy --add-gnu-debuglink=/app/target/release/dial-a-poem.debug /app/target/release/dial-a-poem

RUN curl -sL https://sentry.io/get-cli | bash

RUN mv /app/target/release/dial-a-poem.debug /app
RUN --mount=type=secret,id=sentry_auth_token \
    sentry-cli debug-files upload --include-sources --org sam-wlody --project dial-a-poem --auth-token $(cat /run/secrets/sentry_auth_token) /app/dial-a-poem.debug
RUN rm /app/dial-a-poem.debug

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/dial-a-poem /usr/local/bin

COPY poems.sqlite3 /app/poems.sqlite3
COPY assets/static/ /app/assets/static

ENTRYPOINT ["/usr/local/bin/dial-a-poem"]
