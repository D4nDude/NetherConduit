#!/bin/bash

set -e

echo "Checking formatting"
cargo fmt --all -- --check

echo "Running cargo clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "Running tests"
RUST_BACKTRACE=1 cargo test --workspace --all-features