#!/bin/bash

set -x
set -e

source setup_test.sh

test_workspace_no () {
    #echo "Testing with no_std..."
    (cd sbor; cargo $cargo_cmd $cmd_args --no-default-features --features alloc)
    (cd sbor-tests; cargo $cargo_cmd $cmd_args --no-default-features --features alloc)
    (cd scrypto-abi; cargo $cargo_cmd $cmd_args --no-default-features --features alloc)
    (cd scrypto-tests; cargo $cargo_cmd $cmd_args --no-default-features --features alloc)
    (cd scrypto; cargo $cargo_cmd $cmd_args --no-default-features --features alloc,prelude)
    (cd scrypto; cargo $cargo_cmd $cmd_args --no-default-features --features alloc,prelude --release)
}

test_workspace_sequential () {
    cargo $cargo_cmd $cmd_args -p sbor --no-default-features --features alloc
    cargo $cargo_cmd $cmd_args -p sbor-tests --no-default-features --features alloc
    cargo $cargo_cmd $cmd_args -p scrypto-abi --no-default-features --features alloc
    cargo $cargo_cmd $cmd_args -p scrypto-tests --no-default-features --features alloc
    cargo $cargo_cmd $cmd_args -p scrypto --no-default-features --features alloc,prelude
    cargo $cargo_cmd $cmd_args -p scrypto --no-default-features --features alloc,prelude --release
}

test_workspace_all_in_one() {
    cargo $cargo_cmd $cmd_args \
        -p sbor \
        -p sbor-tests \
        -p scrypto-abi \
        -p scrypto-tests
    cargo $cargo_cmd $cmd_args \
        -p scrypto --no-default-features --features alloc,prelude
    cargo $cargo_cmd $cmd_args \
        -p scrypto  --no-default-features --features alloc,prelude --release
}

$test_cmd
