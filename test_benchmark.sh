#!/bin/bash

rustc_wrapper="${1:-none}"
cargo_cmd="${2:-test}"
cmd_args="${3:-}"

if [ "$rustc_wrapper" != "none" ] ; then
    export RUSTC_WRAPPER=$rustc_wrapper
else
    unset RUSTC_WRAPPER
fi

#echo "Running benchmark..."
cd sbor-tests; cargo bench
cd radix-engine; cargo bench

