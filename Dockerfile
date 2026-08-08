# ── 阶段 1：安装 cargo-chef ────────────────────────────────────────────────────
FROM --platform=$BUILDPLATFORM rust:1.97.1-slim AS chef
ENV CARGO_INCREMENTAL=0 \
    CARGO_NET_GIT_FETCH_WITH_CLI=false \
    CARGO_TERM_COLOR=never
RUN cargo install cargo-chef --locked
WORKDIR /src

# ── 阶段 2：生成依赖缓存计划 ──────────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── 阶段 3：缓存依赖并编译 ────────────────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /src/recipe.json recipe.json
# 仅在 Cargo.lock / 依赖发生变化时重新下载编译
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin locos

# ── 阶段 4：最小化 runtime 镜像 ───────────────────────────────────────────────
FROM debian:trixie-slim AS runtime

RUN groupadd -g 10001 app && useradd -u 10001 -g 10001 --no-create-home --shell /bin/false app

WORKDIR /app

COPY --from=builder /src/target/release/locos locos

USER 10001:10001

ENTRYPOINT ["/app/locos"]
