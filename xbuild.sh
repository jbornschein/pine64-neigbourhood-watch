#!/bin/sh

mkdir -p target-pine64

docker run \
    --rm -t \
    -v $(pwd):/src \
    -v $(pwd)/target-pine64:/src/target \
    -w /src pine64-xenv \
    cargo build --release

