    cargo fuzz build \
        --release \
        --sanitizer=none \
        --no-default-features --features std,libfuzzer-sys \
        --fuzz-dir . wasm
    cp target/x86_64-unknown-linux-gnu/release/wasm $OUT/wasm_fuzzer
