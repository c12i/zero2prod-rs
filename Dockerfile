FROM rust:1.55.0 as builder
WORKDIR /app
COPY ./ ./
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM debian:buster-slim as runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/z2p z2p
COPY configurations configurations
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./z2p"]
