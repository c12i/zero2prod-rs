FROM lukemathwalker/cargo-chef:latest-rust-1.53.0 as chef
WORKDIR /app

FROM chef as planner
COPY ./ ./
# Compute lockfile for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build project dependencies, not the application
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, as long as changes to our depencency tree
# remain the same, all layers should be cached
COPY ./ ./
ENV SQLX_OFFLINE true
RUN cargo build --release --bin z2p

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
