FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
# Compute a lock-lick file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same
# all layers should be cached
COPY . .
ENV SQLX_OFFLINE true
# Build project
RUN cargo build --release --bin z2p

FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/z2p z2p
COPY configurations configurations
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./z2p"]
