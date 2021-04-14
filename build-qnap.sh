#!/bin/env sh

# Path to gcc
LINKER_PATH="$QNAP_TOOLCHAIN_PATH/cross-tools/bin/x86_64-QNAP-linux-gnu-gcc"
# Path to the lib folder
LIB_PATH="$QNAP_TOOLCHAIN_PATH/fs/lib"

export SQLX_OFFLINE=true
export CC=$LINKER_PATH
#Path to ar
export AR="$QNAP_TOOLCHAIN_PATH/cross-tools/bin/x86_64-QNAP-linux-gnu-ar"
cargo rustc "$@" -- "-Clinker=$LINKER_PATH" "-L $LIB_PATH"
