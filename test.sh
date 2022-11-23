#!/bin/bash

set -x
set -e

#test_runner="test"
#cargo help nextest 2>/dev/null >&2 || \
#    test_runner="nextest run"

test_runner=$TEST_RUNNER

cd "$(dirname "$0")"

echo "Testing with std..."
(cd sbor; cargo $test_runner)
(cd sbor-derive; cargo $test_runner)
(cd sbor-tests; cargo $test_runner)
(cd scrypto; cargo $test_runner)
(cd scrypto; cargo $test_runner --release)
(cd scrypto-derive; cargo $test_runner)
(cd scrypto-tests; cargo $test_runner)
(cd radix-engine; cargo $test_runner)
(cd radix-engine; cargo $test_runner --features wasmer)
(cd transaction; cargo $test_runner)

echo "Testing with no_std..."
(cd sbor; cargo $test_runner --no-default-features --features alloc)
(cd sbor-tests; cargo $test_runner --no-default-features --features alloc)
(cd scrypto; cargo $test_runner --no-default-features --features alloc,prelude)
(cd scrypto; cargo $test_runner --no-default-features --features alloc,prelude --release)
(cd scrypto-abi; cargo $test_runner --no-default-features --features alloc)
(cd scrypto-tests; cargo $test_runner --no-default-features --features alloc)

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
