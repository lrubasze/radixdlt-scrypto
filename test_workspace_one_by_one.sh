#!/bin/bash

set -x
set -e

#export RUSTC_WRAPPER=/opt/homebrew/bin/sccache

#test_runner="test"
#cargo help nextest 2>/dev/null >&2 || \
#    test_runner="nextest run"
#
#test_runner="nextest run"
test_runner=$TEST_RUNNER

cd "$(dirname "$0")"

echo "Testing with std..."
(cargo $test_runner -p sbor)
(cargo $test_runner -p sbor-derive)
(cargo $test_runner -p sbor-tests)
(cargo $test_runner -p scrypto)
(cargo $test_runner -p scrypto --release)
(cargo $test_runner -p scrypto-derive)
(cargo $test_runner -p scrypto-tests)
(cargo $test_runner -p radix-engine)
(cargo $test_runner -p radix-engine --features wasmer)
(cargo $test_runner -p transaction)

echo "Testing with no_std..."
(cargo $test_runner -p sbor --no-default-features --features alloc)
(cargo $test_runner -p sbor-tests --no-default-features --features alloc)
(cargo $test_runner -p scrypto --no-default-features --features alloc,prelude)
(cargo $test_runner -p scrypto --no-default-features --features alloc,prelude --release)
(cargo $test_runner -p scrypto-abi --no-default-features --features alloc)
(cargo $test_runner -p scrypto-tests --no-default-features --features alloc)

echo "Building system packages and examples..."
(cd assets/blueprints/account; scrypto test)
(cd assets/blueprints/faucet; scrypto test)
(cd examples/hello-world; scrypto test)
(cd examples/no-std; scrypto test)

echo "Running simulator..."
(cd simulator; bash ./tests/resim.sh)
(cd simulator; bash ./tests/scrypto.sh)
(cd simulator; bash ./tests/manifest.sh)

echo "Running benchmark..."
(cd sbor-tests; cargo bench)
(cd radix-engine; cargo bench)

echo "Congrats! All tests passed."
