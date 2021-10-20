FROM rust:1.55.0 as builder
WORKDIR /app
COPY ./ ./
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM rust:1.55.0 as runtime
WORKDIR /app
COPY --from=builder /app/target/release/z2p z2p
COPY configurations configurations
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./z2p"]
