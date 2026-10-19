ARG RUST_VERSION=1.99.1

# ── 阶段 1：Chef 基础阶段（直接使用官方 cargo-chef 预装镜像）────────────────────
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION}-slim-trixie AS chef
WORKDIR /app
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# 安装 tikv-jemalloc-sys 与 C 扩展编译必需的 make 构建工具；
# webauthn-rs-core -> openssl-sys 需要 pkg-config + libssl-dev 定位系统 OpenSSL
RUN apt-get update \
    && apt-get install -y --no-install-recommends make pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# ── 阶段 2：Planner 依赖分析阶段（生成 recipe.json）───────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── 阶段 3：Builder 预编译与源码构建阶段 ────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# 预编译第三方依赖
RUN cargo chef cook --release --recipe-path recipe.json

# 复制源码并编译最终 Release 二进制程序
COPY . .
RUN cargo build --release --bin locos

# ── 阶段 4：最小化 Runtime 生产运行镜像 ───────────────────────────────────────
FROM debian:trixie-slim AS runtime

ENV LOCO_ENV=production \
    APP_PORT=5150 \
    APP_BINDING=0.0.0.0 \
    TZ=Asia/Shanghai \
    SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -g 10001 app \
    && useradd -u 10001 -g 10001 --no-create-home --shell /bin/false app

WORKDIR /app

# 从 Builder 阶段复制可执行文件、运行时配置文件与国际化资源
COPY --from=builder --chown=app:app /app/target/release/locos /app/locos
COPY --chown=app:app config/ /app/config/
COPY --chown=app:app locales/ /app/locales/

# 预建运行时所需的可写存储目录并授权给非 root 用户
RUN mkdir -p storages/logs storages/uploads storages/tmp && chown -R app:app /app/storages

USER 10001:10001
EXPOSE 5150

ENTRYPOINT ["/app/locos"]
CMD ["start"]
