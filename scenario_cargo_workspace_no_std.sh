#!/bin/bash

# begin
# variants rustc_wrapper=none,sccache;cargo_cmd=build;test_cmd=test_workspace_sequential,test_workspace_all_in_one

# name=Testing with no_std.. (cargo build)
# options clean_before=true
./test_with_no_std.sh
# end

# begin
# variants rustc_wrapper=none,sccache;cargo_cmd=test,nextest run;test_cmd=test_workspace_sequential,test_workspace_all_in_one

# name=Testing with no_std.. build, run (cargo test)
# options clean_before=true
./test_with_no_std.sh
# end

# begin
# variants rustc_wrapper=none,sccache;cargo_cmd=test,nextest run;test_cmd=test_workspace_sequential,test_workspace_all_in_one

# name=Testing with no_std.. build, no run (cargo test --no-run)
# options clean_before=true
./test_with_no_std.sh cmd_args=--no-run

# name=Testing with no_std.. run already built (cargo test)
./test_with_no_std.sh
# end
