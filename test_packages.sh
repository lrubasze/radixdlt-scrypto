#!/bin/bash

set -x
set -e

sourced=0
if [ "$0" != "$BASH_SOURCE" ]; then
    sourced=1
fi

source setup_test.sh

test_packages() {
    #echo "Building system packages and examples..."
    (cd assets/blueprints/account; scrypto test)
    (cd assets/blueprints/faucet; scrypto test)
    (cd examples/hello-world; scrypto test)
    (cd examples/no-std; scrypto test)
}

if [[ $sourced -eq 0 ]] ; then
    test_packages
fi
