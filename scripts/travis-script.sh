#!/bin/bash
set -ex
export RUST_BACKTRACE=1

cargo test
for d in examples/* ; do
    cd "$d"
    cargo test
    cd -
done