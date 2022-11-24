#!/bin/bash

set -x
set -e

source test_with_std.sh
$test_cmd

source test_with_no_std.sh
$test_cmd

source test_packages.sh
test_packages

source test_simulator.sh
test_simulator

source test_benchmark.sh
$test_cmd

echo "Congrats! All tests passed."

