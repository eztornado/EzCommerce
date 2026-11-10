# API EzCommerce (Larust) — build multi-stage.
FROM rust:1-slim AS build
WORKDIR /app

# Dependencias del sistema para sqlx (TLS) y para compilar en modo release.
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache de dependencias: primero los manifests.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs \
    && cargo build --release \
    && rm -rf src

# Código real (app/, routes/, config/, database/ se incluyen via lib.rs con #[path]).
COPY . .
RUN touch src/lib.rs src/main.rs && cargo build --release

# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/ezcommerce-api /usr/local/bin/ezcommerce-api
COPY --from=build /app/database /app/database

# SQLite de desarrollo (en producción se usa MySQL via DB_CONNECTION).
ENV APP_PORT=34187
EXPOSE 34187
CMD ["ezcommerce-api"]
