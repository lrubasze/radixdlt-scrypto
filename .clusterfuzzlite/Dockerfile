FROM gcr.io/oss-fuzz-base/base-builder-rust:v1
RUN apt-get update && apt-get install -y make autoconf automake libtool
COPY . $SRC/fuzz-tests
WORKDIR fuzz-tests
COPY .clusterfuzzlite/build.sh $SRC/
