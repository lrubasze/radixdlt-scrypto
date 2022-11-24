#!/bin/bash

set -x
set -e

sourced=0
if [ "$0" != "$BASH_SOURCE" ]; then
    sourced=1
fi

source setup_test.sh

test_simulator () {
    #echo "Running simulator..."
    (cd simulator; bash ./tests/resim.sh)
    (cd simulator; bash ./tests/scrypto.sh)
    (cd simulator; bash ./tests/manifest.sh)
}

if [[ $sourced -eq 0 ]] ; then
    test_simulator
fi
