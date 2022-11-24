#!/bin/bash

set -x
set -e

source setup_test.sh

test_workspace_no () {
    (cd sbor; cargo $cargo_cmd $cmd_args)
    (cd sbor-derive; cargo $cargo_cmd $cmd_args)
    (cd sbor-tests; cargo $cargo_cmd $cmd_args)
    (cd scrypto; cargo $cargo_cmd $cmd_args)
    (cd scrypto-derive; cargo $cargo_cmd $cmd_args)
    (cd scrypto-tests; cargo $cargo_cmd $cmd_args)
    (cd radix-engine; cargo $cargo_cmd $cmd_args)
    (cd transaction; cargo)
    (cd scrypto; cargo $cargo_cmd $cmd_args --release)
    (cd radix-engine; cargo $cargo_cmd $cmd_args --features wasmer)
}

test_workspace_sequential () {
    cargo $cargo_cmd $cmd_args -p sbor
    cargo $cargo_cmd $cmd_args -p sbor-derive
    cargo $cargo_cmd $cmd_args -p sbor-tests
    cargo $cargo_cmd $cmd_args -p scrypto
    cargo $cargo_cmd $cmd_args -p scrypto-derive
    cargo $cargo_cmd $cmd_args -p scrypto-tests
    cargo $cargo_cmd $cmd_args -p radix-engine
    cargo $cargo_cmd $cmd_args -p transaction
    cargo $cargo_cmd $cmd_args -p scrypto --release
    cargo $cargo_cmd $cmd_args -p radix-engine --features wasmer
}

test_workspace_all_in_one() {
    cargo $cargo_cmd $cmd_args \
        -p sbor \
        -p sbor-derive \
        -p sbor-tests \
        -p scrypto \
        -p scrypto-derive \
        -p scrypto-tests \
        -p radix-engine \
        -p transaction
    cargo $cargo_cmd $cmd_args \
        -p scrypto --release
    cargo $cargo_cmd $cmd_args \
        -p radix-engine --features wasmer
}

$test_cmd
