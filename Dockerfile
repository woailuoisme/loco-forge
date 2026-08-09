# ── 阶段 1：编译二进制 ────────────────────────────────────────────────────────
FROM --platform=$BUILDPLATFORM rust:1.97.1-slim AS builder
ENV CARGO_INCREMENTAL=0
RUN apt-get update \
    && apt-get install -y --no-install-recommends lld \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN cargo build --release --bin locos

# ── 阶段 2：最小化 runtime 镜像 ───────────────────────────────────────────────
FROM debian:trixie-slim AS runtime

ENV LOCO_ENV=production \
    PORT=5150 \
    BINDING=0.0.0.0 \
    TZ=Asia/Shanghai \
    SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata wget \
    && ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -g 10001 app \
    && useradd -u 10001 -g 10001 --no-create-home --shell /bin/false app

WORKDIR /app

COPY --from=builder /src/target/release/locos .
COPY config/ config/

RUN mkdir -p storages/logs storages/uploads && chown -R app:app /app

USER 10001:10001

EXPOSE 5150

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -qO- http://127.0.0.1:5150/health || exit 1

ENTRYPOINT ["./locos"]
CMD ["start"]


