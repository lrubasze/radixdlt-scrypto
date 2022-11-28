#!/bin/bash

# begin
# variants rustc_wrapper=none,sccache;test_cmd=test_workspace_no,test_workspace_sequential,test_workspace_all_in_one

# name=Testing benchmark
# options clean_before=true
./test_benchmark.sh
# end

