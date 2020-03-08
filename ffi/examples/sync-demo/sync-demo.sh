#!/bin/bash

#
# Compiles and runs alass-ffi example cli utility
#
# Must create release build before running:
#  `cargo build --release`
#

#LIB_TYPE=static
LIB_TYPE=dynamic

function abort() {
  echo "Unable to find build artifacts. Make sure the alass-ffi crate has been built: cargo build --release"
  exit 1
}

DEMO_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BUILD_DIR=$DEMO_DIR/build
TARGET_DIR=$DEMO_DIR/../../../target/release

[[ ! -d $TARGET_DIR ]] && abort

HEADER_PATH=$(find $TARGET_DIR -type f -name 'alass.h' | xargs ls -1t | head -n 1)
LIB_PATH=$(find $TARGET_DIR -type f -name libalass.a | head -n 1)
DYLIB_PATH=$(find -E $TARGET_DIR -type f -regex ".*\libalass\.(so|dylib|dll)" | head -n 1)

[[ ! -f $HEADER_PATH || ! -f $LIB_PATH || ! -f $DYLIB_PATH ]] && abort

mkdir -p $BUILD_DIR

HEADER_DIR=$(dirname $HEADER_PATH)

if [[ $LIB_TYPE =~ static ]]; then
  gcc $DEMO_DIR/sync-demo.c -I $HEADER_DIR -o $BUILD_DIR/sync-demo $LIB_PATH
elif [[ $LIB_TYPE =~ dynamic ]]; then
  gcc $DEMO_DIR/sync-demo.c -I $HEADER_DIR -o $BUILD_DIR/sync-demo $DYLIB_PATH
fi

[[ $? == 0 ]] && $BUILD_DIR/sync-demo $@
