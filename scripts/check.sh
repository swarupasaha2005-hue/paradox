#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
npm run lint
npm run typecheck
npm run build
cargo fmt --all -- --check
cargo check --workspace --locked
stellar contract build --locked
