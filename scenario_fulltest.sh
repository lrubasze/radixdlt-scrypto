#!/bin/bash

# begin
# variants rustc_wrapper=sccache;cargo_cmd=nextest run;test_cmd=test_workspace_sequential,test_workspace_all_in_one

# options clean_before=true
# name=Fulltest
./test_fulltest.sh

# end
