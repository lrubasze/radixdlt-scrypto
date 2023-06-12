
## info
https://google.github.io/clusterfuzzlite/build-integration/

It works only on Linux (even if running in docker)

export PATH_TO_PROJECT=/work/rdx/radixdlt-scrypto

## build
python infra/helper.py build_image --external $PATH_TO_PROJECT
python infra/helper.py build_fuzzers --external $PATH_TO_PROJECT --sanitizer none

## check
python infra/helper.py check_build --external $PATH_TO_PROJECT --sanitizer none

## run
python infra/helper.py run_fuzzer --external --sanitizer=none --corpus-dir=corpus/transaction $PATH_TO_PROJECT transaction_fuzzer -- -jobs=4 -fork=1 -ignore_crashes=1 -max_total_time=26000

above command clears corpus dir, so using below

docker run --rm --privileged --shm-size=2g --platform linux/amd64 -i -t \
    -e FUZZING_ENGINE=libfuzzer -e SANITIZER=none \
    -e RUN_FUZZER_MODE=interactive -e HELPER=True \
    -v /work/rdx/oss-fuzz/corpus/transaction:/tmp/transaction_fuzzer_corpus \
    -v /work/rdx/oss-fuzz/build/out/radixdlt-scrypto:/out \
    gcr.io/oss-fuzz-base/base-runner \
    bash -c "cd /out; /out/transaction_fuzzer -rss_limit_mb=2560 -timeout=25 -jobs=4 -fork=1 -ignore_crashes=1 -max_total_time=26000 /tmp/transaction_fuzzer_corpus"


## build for coverage
python infra/helper.py build_fuzzers --external --sanitizer coverage $PATH_TO_PROJECT

## get coverage
python infra/helper.py coverage --external $PATH_TO_PROJECT --fuzz-target=transaction_fuzzer --corpus-dir=corpus/transaction

# there is some issue with this file
error: /out/rustc/871b5952023139738f72eba235063575062bc2e9/library/std/src/sys/common/thread_local/fast_local.rs: No such file or directory
warning: The file '/rustc/871b5952023139738f72eba235063575062bc2e9/library/std/src/sys/common/thread_local/fast_local.rs' isn't covered.

workaround:
- find it in build/out in another folder and copy it to the expected location
- exclude from coverage analysis
