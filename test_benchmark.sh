#!/bin/bash

set -x
set -e

sourced=0
if [ "$0" != "$BASH_SOURCE" ]; then
    sourced=1
fi

source setup_test.sh

test_workspace_no () {
    #echo "Running benchmark..."
    (cd sbor-tests; cargo bench)
    (cd radix-engine; cargo bench)
}

test_workspace_sequential () {
    cargo bench -p sbor-tests
    cargo bench -p radix-engine
}

test_workspace_all_in_one() {
    cargo bench \
        -p sbor-tests \
        -p radix-engine
}

if [[ $sourced -eq 0 ]] ; then
    $test_cmd
fi
