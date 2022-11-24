
# test params might be provided as:
# - environmental variables
# - key=value arguments
# - if not provided then defaults are used
eval $@

# set default values if not available
test_cmd=${test_cmd:-test_workspace_no}
rustc_wrapper=${rustc_wrapper:-none}
cargo_cmd="${cargo_cmd:-test}"
cmd_args="${cmd_args:-}"

if [ "$rustc_wrapper" != "none" ] ; then
    export RUSTC_WRAPPER=$rustc_wrapper
else
    unset RUSTC_WRAPPER
fi
