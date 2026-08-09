ARG CARGO_CHEF_VERSION=v0.1.77

# ── 阶段 1：Chef 基础阶段（安装 cargo-chef 预编译二进制）──────────────────────
FROM rust:1.97-slim-trixie AS chef
ARG CARGO_CHEF_VERSION
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends curl xz-utils \
    && curl --proto '=https' --tlsv1.2 -LsSf https://github.com/LukeMathWalker/cargo-chef/releases/download/${CARGO_CHEF_VERSION}/cargo-chef-installer.sh | sh \
    && rm -rf /var/lib/apt/lists/*

# ── 阶段 2：Planner 依赖分析阶段（生成 recipe.json）───────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── 阶段 3：Builder 预编译与源码构建阶段 ────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /src/recipe.json recipe.json
# 缓存第三方依赖编译产物，只有 Cargo.toml / Cargo.lock 变动时才会重新编译
RUN cargo chef cook --release --recipe-path recipe.json
# 复制源码并编译最终二进制
COPY . .
RUN cargo build --release --bin locos

# ── 阶段 4：最小化 runtime 镜像 ───────────────────────────────────────────────
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

HEALTHCHECK --interval=60s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -qO- http://127.0.0.1:5150/health || exit 1

ENTRYPOINT ["./locos"]
CMD ["start"]
