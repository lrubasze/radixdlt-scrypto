#!/bin/bash

source setup_test.sh

#echo "Building system packages and examples..."
cd assets/blueprints/account; scrypto test
cd assets/blueprints/faucet; scrypto test
cd examples/hello-world; scrypto test
cd examples/no-std; scrypto test
