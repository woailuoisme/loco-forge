# ==============================================================================
# 文件作用：Debian (Glibc) 多阶段高性能容器镜像构建规范
# 生效时机：执行 `docker build -f Dockerfile .` 或生产 CI/CD 流水线时执行
# 核心职责：
#   1. 多阶段构建，分离编译环境与运行环境，减小最终镜像体积
#   2. 采用 Debian Slim 作为极简 Runtime，兼顾 Glibc 极致性能与基础环境兼容性
#   3. 配置非 Root 最小权限运行用户 (UID 10001)、时区与健康检查自愈探针
# ==============================================================================

# ── 全局构建参数 ─────────────────────────────────────────────────────────────
ARG RUST_VERSION=1.97.1

# ── 阶段 1：Builder 源码构建阶段 ─────────────────────────────────────────────
FROM rust:${RUST_VERSION}-slim-trixie AS builder
WORKDIR /src
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# 复制源码并编译最终 Release 二进制程序
COPY . .
RUN cargo build --release --bin locos

# ── 阶段 2：最小化 Runtime 生产运行镜像 ───────────────────────────────────────
FROM debian:trixie-slim AS runtime

ENV LOCO_ENV=production \
    PORT=5150 \
    BINDING=0.0.0.0 \
    TZ=Asia/Shanghai \
    SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata wget \
    && ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -g 10001 app \
    && useradd -u 10001 -g 10001 --no-create-home --shell /bin/false app

WORKDIR /app

# 从 Builder 阶段复制可执行文件与运行时配置文件
COPY --from=builder /src/target/release/locos /app/locos
COPY config/ /app/config/

# 预建运行时所需的可写存储目录并授权给非 root 用户
RUN mkdir -p storages/logs storages/uploads storages/tmp && chown -R app:app /app

USER 10001:10001
EXPOSE 5150

# 容器健康检查自愈探针
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -qO- http://127.0.0.1:5150/health || exit 1

ENTRYPOINT ["/app/locos"]
CMD ["start"]
