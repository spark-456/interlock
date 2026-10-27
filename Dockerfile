# Interlock: web build → Rust release (web embedded) → slim runtime with ffmpeg.
FROM node:22-slim AS web
WORKDIR /src/web
RUN corepack enable && corepack prepare pnpm@10.14.0 --activate
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY web/ ./
RUN pnpm build

FROM rust:1-slim-bookworm AS rust
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config build-essential && rm -rf /var/lib/apt/lists/*
COPY . .
COPY --from=web /src/web/build ./web/build
RUN cargo build --release -p il-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ffmpeg ca-certificates curl && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 interlock && mkdir /data && chown interlock /data
COPY --from=rust /src/target/release/il-server /usr/local/bin/il-server
COPY config/interlock.example.toml /etc/interlock/interlock.toml
COPY fixtures/templates /fixtures/templates
COPY fixtures/recorded /fixtures/recorded
COPY fixtures/apis /fixtures/apis
WORKDIR /
USER interlock
VOLUME /data
ENV IL_HOST=0.0.0.0 IL_PORT=8080 IL_CONFIG=/etc/interlock/interlock.toml IL_LOG_JSON=1 IL_PROVIDERS=real
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=30s CMD curl --fail --silent http://127.0.0.1:8080/healthz > /dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/il-server"]
