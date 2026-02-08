#!/usr/bin/env bash
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive
export TZ=Asia/Seoul

rustup toolchain install 1.93.0
rustup default 1.93.0