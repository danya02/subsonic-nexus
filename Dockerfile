# --- build stage ---
FROM rustlang/rust:nightly-2026-05-28-bookworm AS builder

RUN apt-get update && apt-get install -y libsqlite3-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/subsonic-nexus /subsonic-nexus

# --- runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends libsqlite3-0 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /subsonic-nexus /app/subsonic-nexus

# nexus.toml is expected at /app/nexus.toml (mount a ConfigMap/Secret here)
# nexus.db is expected at /app/nexus.db   (mount a PVC here)

EXPOSE 3000

ENTRYPOINT ["/app/subsonic-nexus"]
