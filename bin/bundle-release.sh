#!/bin/bash

#
# Packages target-specific build artifacts into bundle for Github Action
# release workflow. Must be run from root project folder.
#
# Expected envars:
#  - TARGET:   rustc build target
#  - PLATFORM: linux, macos, or windows
#

VERSION=$(awk -F'"' '/^version/ {print $2}' ffi/Cargo.toml | head -n 1)
BUNDLE_NAME=alass-ffi-$VERSION-$TARGET
BUILD_DIR=target/$TARGET/release
BUNDLE_DIR=dist/$BUNDLE_NAME

if [[ $PLATFORM =~ linux ]]; then
  LINUX=true
  LIB_STATIC=libalass.a
  LIB_DYNAMIC=libalass.so
  BUNDLE_EXT=tar.gz
elif [[ $PLATFORM =~ macos ]]; then
  MACOS=true
  LIB_STATIC=libalass.a
  LIB_DYNAMIC=libalass.dylib
  BUNDLE_EXT=tar.gz
  [[ $TARGET =~ ios ]] && IOS=true
elif [[ $PLATFORM =~ windows ]]; then
  WINDOWS=true
  LIB_STATIC=alass.lib
  LIB_DYNAMIC=alass.dll
  BUNDLE_EXT=zip
fi

BUNDLE_FILE=$BUNDLE_NAME.$BUNDLE_EXT
BUNDLE_PATH=dist/$BUNDLE_FILE

# Create output directory
mkdir -p $BUNDLE_DIR

# Include generated header
cp $(find $BUILD_DIR -type f -name 'alass.h' | xargs ls -1t | head -n 1) $BUNDLE_DIR

# Include static lib (except for MacOS universal)
if [[ $PLATFORM != macos && $TARGET != universal-apple-darwin ]]; then
  cp $BUILD_DIR/$LIB_STATIC $BUNDLE_DIR
fi

# Look for MacOS Universal dylib in the root directory
[[ $PLATFORM =~ macos && $TARGET =~ universal ]] && BUILD_DIR=.

# Include dynamic lib
[[ ! $IOS ]] && cp $BUILD_DIR/$LIB_DYNAMIC $BUNDLE_DIR

# Make dynamic lib 'id' relative to @rpath on MacOS
[[ $PLATFORM =~ macos && -f $BUNDLE_DIR/$LIB_DYNAMIC ]] &&
  install_name_tool -id "@rpath/$LIB_DYNAMIC" $BUNDLE_DIR/$LIB_DYNAMIC

# Generate bundle file
if [[ $WINDOWS ]]; then
  (cd $BUNDLE_DIR && 7z a -tzip ../$BUNDLE_FILE *)
else
  (cd dist && tar -czf $BUNDLE_FILE $BUNDLE_NAME)
fi

# Log bundle checksum
if [[ $WINDOWS ]]; then
  (cd dist && certutil -hashfile $BUNDLE_FILE SHA256)
else
  echo "Bundle checksum (SHA-256):"
  (cd dist && shasum -a 256 $BUNDLE_FILE)
fi

# Export downstream Github Action variables
echo ::set-output name=bundle_file::$BUNDLE_FILE
echo ::set-output name=bundle_path::$BUNDLE_PATH
