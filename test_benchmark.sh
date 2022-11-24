#!/bin/bash

source setup_test.sh

#echo "Running benchmark..."
cd sbor-tests; cargo bench
cd radix-engine; cargo bench

