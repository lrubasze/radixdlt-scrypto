#!/bin/bash

rustc_wrapper="${1:-none}"
cargo_cmd="${2:-test}"
cmd_args="${3:-}"

if [ "$rustc_wrapper" != "none" ] ; then
    export RUSTC_WRAPPER=$rustc_wrapper
else
    unset RUSTC_WRAPPER
fi

#echo "Building system packages and examples..."
cd assets/blueprints/account; scrypto test
cd assets/blueprints/faucet; scrypto test
cd examples/hello-world; scrypto test
cd examples/no-std; scrypto test
