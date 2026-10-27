# Interlock: web build → Rust release (web embedded) → slim runtime with ffmpeg. Finalised in R02.
FROM node:22-slim AS web
WORKDIR /src/web
RUN corepack enable
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
RUN apt-get update && apt-get install -y --no-install-recommends ffmpeg ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 interlock && mkdir /data && chown interlock /data
COPY --from=rust /src/target/release/il-server /usr/local/bin/il-server
COPY config/interlock.example.toml /etc/interlock/interlock.toml
USER interlock
VOLUME /data
ENV IL_HOST=0.0.0.0 IL_PORT=8080 IL_CONFIG=/etc/interlock/interlock.toml IL_LOG_JSON=1
EXPOSE 8080
HEALTHCHECK CMD ["/bin/sh", "-c", "exec 3<>/dev/tcp/127.0.0.1/8080"]
ENTRYPOINT ["/usr/local/bin/il-server"]
