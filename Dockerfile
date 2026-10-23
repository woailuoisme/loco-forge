ARG RUST_VERSION=1.99.1

# ── Chef 基础阶段 ─────────────────────────────────────────────────────────────
# 非 slim 变体基于 buildpack-deps，自带 gcc / make / perl；openssl 走 vendored
# 源码编译，故无需额外安装任何包
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# ── Planner：生成依赖清单 recipe.json ──────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Builder：预编译依赖 + 构建 Release 二进制 ─────────────────────────────────
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# recipe 变更时 registry 缓存避免重下 crate
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef cook --release --locked --bin locos --recipe-path recipe.json

COPY . .
RUN cargo build --release --locked --bin locos

# ── Runtime：最小化生产镜像 ───────────────────────────────────────────────────
FROM debian:trixie-slim AS runtime

ENV LOCO_ENV=production \
    APP_PORT=5150 \
    APP_BINDING=0.0.0.0 \
    TZ=Asia/Shanghai

# slim 镜像不含 CA 证书与时区库，出站 TLS / 时区依赖它们
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复用镜像内置 nobody:nogroup，不自建账号
COPY --from=builder --chown=nobody:nogroup /app/target/release/locos /app/locos
COPY --chown=nobody:nogroup config/ /app/config/
COPY --chown=nobody:nogroup locales/ /app/locales/

RUN mkdir -p storages/logs storages/uploads storages/tmp && chown -R nobody:nogroup /app/storages

USER nobody:nogroup
EXPOSE 5150

ENTRYPOINT ["/app/locos"]
CMD ["start"]
