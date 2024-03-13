FROM gcr.io/oss-fuzz-base/base-builder-rust:v1
RUN apt-get update && apt-get install -y make autoconf automake libtool
RUN rustup default nightly-2023-03-04
RUN rustup target add wasm32-unknown-unknown --toolchain=nightly-2023-03-04
COPY . $SRC/radix
COPY .clusterfuzzlite/build.sh $SRC/
WORKDIR radix
