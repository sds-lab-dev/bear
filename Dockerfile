ARG CARGO_PROFILE=release
ARG CARGO_REGISTRY_CACHE_ID=cargo-registry-cache
ARG CARGO_GIT_CACHE_ID=cargo-git-cache
ARG CARGO_APP_CACHE_ID=cargo-app-cache
ARG CARGO_TARGET_DIR=/var/tmp/target

#
# Install cargo-chef
#
FROM rust:1.93-alpine AS chef
WORKDIR /usr/src/app
RUN apk add --no-cache \
    musl-dev \
    gcc \
    libc-dev \
    make \
    linux-headers \
    openssl-dev \
    openssl-libs-static \
    git
RUN cargo install cargo-chef

#
# Extract dependencies information
#
FROM chef AS planner
WORKDIR /usr/src/app
COPY . .
RUN cargo chef prepare \
    --recipe-path recipe.json

#
# Build dependencies
#
FROM chef AS dependency-builder
WORKDIR /usr/src/app
ARG CARGO_REGISTRY_CACHE_ID
ARG CARGO_GIT_CACHE_ID
ARG CARGO_APP_CACHE_ID
ARG CARGO_TARGET_DIR
ARG CARGO_PROFILE
COPY --from=planner /usr/src/app/recipe.json recipe.json
# Build all dependencies from workspace for caching (/usr/local/cargo is the default RUST_HOME)
RUN --mount=type=cache,id=${CARGO_REGISTRY_CACHE_ID},target=/usr/local/cargo/registry \
    --mount=type=cache,id=${CARGO_GIT_CACHE_ID},target=/usr/local/cargo/git \
    --mount=type=cache,id=${CARGO_APP_CACHE_ID},target=${CARGO_TARGET_DIR} \
    cargo chef cook \
    --recipe-path recipe.json \
    --profile ${CARGO_PROFILE} \
    --workspace \
    --bins \
    --target-dir ${CARGO_TARGET_DIR}

#
# Build application
#
FROM chef AS app-builder
WORKDIR /usr/src/app
ARG CARGO_REGISTRY_CACHE_ID
ARG CARGO_GIT_CACHE_ID
ARG CARGO_APP_CACHE_ID
ARG CARGO_TARGET_DIR
ARG CARGO_PROFILE
COPY . .
RUN --mount=type=cache,id=${CARGO_REGISTRY_CACHE_ID},target=/usr/local/cargo/registry \
    --mount=type=cache,id=${CARGO_GIT_CACHE_ID},target=/usr/local/cargo/git \
    --mount=type=cache,id=${CARGO_APP_CACHE_ID},target=${CARGO_TARGET_DIR} \
    cargo build \
    --profile ${CARGO_PROFILE} \
    --manifest-path Cargo.toml \
    --target-dir ${CARGO_TARGET_DIR} && \
    PROFILE_DIR=$( [ "${CARGO_PROFILE}" = "dev" ] && echo 'debug' || echo "${CARGO_PROFILE}" ) && \
    cp "${CARGO_TARGET_DIR}/${PROFILE_DIR}/bear" /usr/local/bin

#
# Final image for bear application
#
FROM alpine:3.22 AS bear-app

ARG CARGO_TARGET_DIR

RUN apk add --no-cache \
    ca-certificates \
    openssl

COPY --from=app-builder /usr/local/bin/bear /usr/local/bin

ENTRYPOINT ["/usr/local/bin/bear"]