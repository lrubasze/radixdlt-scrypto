#!/bin/bash -eu

if [ -d $WORK/target ] ; then
    cp -dprf $WORK/target .
fi
cargo +nightly fuzz build \
    --release \
    --no-default-features --features std,libfuzzer-sys \
    --fuzz-dir . \
    --no-cfg-fuzzing transaction

cp -dprf target $WORK
find target -name transaction

cp target/x86_64-unknown-linux-gnu/release/transaction $OUT/transaction_fuzzer
