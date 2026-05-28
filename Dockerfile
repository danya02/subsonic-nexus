# --- build stage ---
FROM rust:1.87-bookworm AS builder

RUN apt-get update && apt-get install -y libsqlite3-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release

# --- runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends libsqlite3-0 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/subsonic-nexus /app/subsonic-nexus

# nexus.toml is expected at /app/nexus.toml (mount a ConfigMap/Secret here)
# nexus.db is expected at /app/nexus.db   (mount a PVC here)

EXPOSE 3000

ENTRYPOINT ["/app/subsonic-nexus"]
