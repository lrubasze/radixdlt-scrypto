#!/bin/bash

rustc_wrapper="${1:-none}"
cargo_cmd="${2:-test}"
cmd_args="${3:-}"

if [ "$rustc_wrapper" != "none" ] ; then
    export RUSTC_WRAPPER=$rustc_wrapper
else
    unset RUSTC_WRAPPER
fi

#echo "Running simulator..."
cd simulator; bash ./tests/resim.sh
cd simulator; bash ./tests/scrypto.sh
cd simulator; bash ./tests/manifest.sh

