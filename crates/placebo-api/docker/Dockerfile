# Stage 1: cargo-chef planner
FROM rust:1.83-bookworm AS planner
RUN cargo install cargo-chef --locked
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: cargo-chef cook (dependency caching)
FROM rust:1.83-bookworm AS cacher
RUN cargo install cargo-chef --locked
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: build the actual binary
FROM rust:1.83-bookworm AS builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin placebo-api

# Stage 4: runtime
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    libgeos-c1v5 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash placebo
USER placebo
WORKDIR /home/placebo

COPY --from=builder /app/target/release/placebo-api /usr/local/bin/placebo-api

EXPOSE 3001

ENTRYPOINT ["placebo-api"]
