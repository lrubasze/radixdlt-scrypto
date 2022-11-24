#!/bin/bash

# begin
# variants rustc_wrapper=none,sccache;cargo_cmd=test,nextest run

# options clean_before=true
# name=Testing with std..
./test_with_std.sh

# name=Testing with no_std..
./test_with_no_std.sh
# end

# begin
# variants rustc_wrapper=none,sccache

# name=Building system packages and examples...
./test_packages.sh

# name=Running simulator...
./test_simulator.sh

# name=Running benchmark...
./test_benchmark.sh
# end

#### Additional tests

# begin
# variants rustc_wrapper=none,sccache;cargo_cmd=build

# name=Testing with std.. (cargo build)"
# options clean_before=true
./test_with_std.sh
# end

# begin
# variants rustc_wrapper=none,sccache;cargo_cmd=test,nextest run

# name=Testing with std.. (no-run)
# options clean_before=true
./test_with_std.sh cargo_args=--no-run

# name=Testing with std.. (run already built)
./test_with_std.sh
# end
