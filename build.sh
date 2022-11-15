#!/bin/bash

export RUST_LOG=debug
export RUSTFLAGS="-C target-cpu=native -g"

echo "cargo build --release"
cargo build --release

echo "cargo test --package cores_ext --lib -- tests::test_cores_ext --exact --nocapture"
cargo test --package cores_ext --lib -- tests::test_cores_ext --exact --nocapture
