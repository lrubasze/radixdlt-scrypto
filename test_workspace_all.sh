#!/bin/bash

#set -x
set -e

#test_runner="test -- --list"
test_runner="test"
#nextest
test_runner=$TEST_RUNNER

packages_with_std=(
"sbor"
"sbor-derive"
"sbor-tests"
"scrypto"
"scrypto-derive"
"scrypto-tests"
"radix-engine"
"transaction"
)

packages_with_no_std=(
"sbor"
"sbor-tests"
"scrypto-abi"
"scrypto-tests"
)

generate_args () {
    local pckgs=("$@")
    local args=""
    for p in "${pckgs[@]}" ; do
        args+="-p $p "
        #echo $p
    done
    echo $args
}


echo "Testing with std..."
(cargo $test_runner \
    -p sbor \
    -p sbor-derive \
    -p sbor-tests \
    -p scrypto \
    -p scrypto-derive \
    -p scrypto-tests \
    -p radix-engine \
    -p transaction)
(cargo $test_runner \
    -p scrypto --release)
(cargo $test_runner \
    -p radix-engine --features wasmer)

echo "Testing with no_std..."
(cargo $test_runner \
    -p sbor \
    -p sbor-tests \
    -p scrypto-abi \
    -p scrypto-tests)
(cargo $test_runner \
    -p scrypto --no-default-features --features alloc,prelude)
(cargo $test_runner \
    -p scrypto --no-default-features --features alloc,prelude --release)

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
